//! Checkout and sale history. `checkout` is the one place stock leaves
//! for a sale — see docs/database.md's "never update current_stock
//! outside a stock_movements insert in the same transaction" rule, and
//! docs/synchronization.md for the idempotency-key contract this
//! implements (the first real consumer; Phase 14's offline queue reuses
//! it unchanged).

pub mod error;
pub use error::SalesError;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::db::repositories::{
    audit_logs, discounts, products, sales as repo, stock_movements, taxes,
};
use crate::db::{DbError, DbPool};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CartLineInput {
    pub product_id: i64,
    pub quantity: i64,
    /// A manual line discount (integer minor units) the cashier applies,
    /// overriding the product's own assigned discount. `None` falls back
    /// to auto-applying the product's discount, if it has one.
    pub discount_override: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentInput {
    pub method: String,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutInput {
    pub customer_id: Option<i64>,
    pub items: Vec<CartLineInput>,
    pub payments: Vec<PaymentInput>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaleItemDto {
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: i64,
    pub discount: i64,
    pub tax: i64,
    pub line_total: i64,
}

#[derive(Debug, Serialize)]
pub struct PaymentDto {
    pub method: String,
    pub amount: i64,
    pub received_at: String,
}

#[derive(Debug, Serialize)]
pub struct SaleDto {
    pub id: String,
    pub invoice_number: String,
    pub customer_id: Option<i64>,
    pub cashier_name: String,
    pub subtotal: i64,
    pub discount_total: i64,
    pub tax_total: i64,
    pub total: i64,
    pub status: String,
    pub created_at: String,
    pub items: Vec<SaleItemDto>,
    pub payments: Vec<PaymentDto>,
}

#[derive(Debug, Serialize)]
pub struct SaleSummaryDto {
    pub id: String,
    pub invoice_number: String,
    pub cashier_name: String,
    pub total: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResult {
    pub sale: SaleDto,
    /// Total paid minus the sale total. Not persisted as its own column
    /// — always derivable as `sum(payments.amount) - sale.total` — but
    /// returned here so the receipt can show it immediately.
    pub change: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct SaleListQuery {
    pub status: Option<String>,
}

const VALID_PAYMENT_METHODS: &[&str] = &["cash", "card", "bank_transfer", "other"];

pub fn checkout(
    pool: &DbPool,
    user: &AuthenticatedUser,
    input: CheckoutInput,
) -> Result<CheckoutResult, SalesError> {
    if input.items.is_empty() {
        return Err(SalesError::Validation(
            "A sale must have at least one item".into(),
        ));
    }
    for line in &input.items {
        if line.quantity <= 0 {
            return Err(SalesError::Validation(
                "Quantity must be greater than zero".into(),
            ));
        }
    }
    if input.payments.is_empty() {
        return Err(SalesError::Validation(
            "A sale must have at least one payment".into(),
        ));
    }
    for payment in &input.payments {
        if !VALID_PAYMENT_METHODS.contains(&payment.method.as_str()) {
            return Err(SalesError::Validation(format!(
                "Unknown payment method: {}",
                payment.method
            )));
        }
        if payment.amount <= 0 {
            return Err(SalesError::Validation(
                "Payment amounts must be greater than zero".into(),
            ));
        }
    }

    let mut conn = pool.get().map_err(DbError::from)?;

    if let Some(key) = &input.idempotency_key {
        if let Some(existing_id) = repo::find_by_idempotency_key(&conn, key)? {
            let sale = load_sale(&conn, &existing_id)?.ok_or(SalesError::SaleNotFound)?;
            let paid: i64 = sale.payments.iter().map(|p| p.amount).sum();
            return Ok(CheckoutResult {
                change: paid - sale.total,
                sale,
            });
        }
    }

    let tx = conn.transaction()?;

    struct ComputedLine {
        product_id: i64,
        quantity: i64,
        unit_price: i64,
        discount: i64,
        tax: i64,
        line_total: i64,
    }

    let mut computed_lines = Vec::with_capacity(input.items.len());
    let mut subtotal = 0i64;
    let mut discount_total = 0i64;
    let mut tax_total = 0i64;

    for line in &input.items {
        let product =
            products::find_by_id(&tx, line.product_id)?.ok_or(SalesError::ProductNotFound)?;
        if !product.is_active {
            return Err(SalesError::Validation(format!(
                "{} is not active",
                product.name
            )));
        }
        if product.current_stock < line.quantity {
            return Err(SalesError::InsufficientStock {
                product_name: product.name,
                available: product.current_stock,
                requested: line.quantity,
            });
        }

        let unit_price = product.selling_price;
        let line_subtotal = unit_price * line.quantity;

        let discount = match line.discount_override {
            Some(amount) => amount.clamp(0, line_subtotal),
            None => {
                compute_product_discount(&tx, product.discount_id, line.quantity, line_subtotal)?
            }
        };

        let tax = compute_product_tax(&tx, product.tax_id, line_subtotal - discount)?;
        let line_total = line_subtotal - discount + tax;

        subtotal += line_subtotal;
        discount_total += discount;
        tax_total += tax;

        computed_lines.push(ComputedLine {
            product_id: product.id,
            quantity: line.quantity,
            unit_price,
            discount,
            tax,
            line_total,
        });
    }

    let total = subtotal - discount_total + tax_total;

    let paid: i64 = input.payments.iter().map(|p| p.amount).sum();
    if paid < total {
        return Err(SalesError::Validation(format!(
            "Payments ({paid}) do not cover the total ({total})"
        )));
    }

    let sale_id = uuid::Uuid::new_v4().to_string();
    let invoice_number = repo::next_invoice_number(&tx)?;

    repo::insert_sale(
        &tx,
        &sale_id,
        &invoice_number,
        input.customer_id,
        user.id,
        subtotal,
        discount_total,
        tax_total,
        total,
        input.idempotency_key.as_deref(),
    )?;

    for line in &computed_lines {
        repo::insert_sale_item(
            &tx,
            &sale_id,
            line.product_id,
            line.quantity,
            line.unit_price,
            line.discount,
            line.tax,
            line.line_total,
        )?;
        products::adjust_stock(&tx, line.product_id, -line.quantity)?;
        stock_movements::insert(
            &tx,
            line.product_id,
            -line.quantity,
            "sale",
            Some("sale"),
            Some(&sale_id),
            Some(user.id),
        )?;
    }

    for payment in &input.payments {
        repo::insert_payment(&tx, &sale_id, &payment.method, payment.amount)?;
    }

    audit_logs::insert(
        &tx,
        Some(user.id),
        None,
        "SALE_CREATED",
        "sale",
        &sale_id,
        None,
        Some(&format!(
            r#"{{"invoice_number":"{invoice_number}","total":{total}}}"#
        )),
        None,
    )?;

    tx.commit()?;

    let sale = load_sale(&conn, &sale_id)?.ok_or(SalesError::SaleNotFound)?;
    Ok(CheckoutResult {
        change: paid - total,
        sale,
    })
}

pub fn list(pool: &DbPool, query: SaleListQuery) -> Result<Vec<SaleSummaryDto>, SalesError> {
    let conn = pool.get().map_err(DbError::from)?;
    let filter = repo::SaleFilter {
        status: query.status,
        limit: 100,
    };
    Ok(repo::list_sales(&conn, &filter)?
        .into_iter()
        .map(|s| SaleSummaryDto {
            id: s.id,
            invoice_number: s.invoice_number,
            cashier_name: s.cashier_name,
            total: s.total,
            status: s.status,
            created_at: s.created_at,
        })
        .collect())
}

pub fn get(pool: &DbPool, id: &str) -> Result<SaleDto, SalesError> {
    let conn = pool.get().map_err(DbError::from)?;
    load_sale(&conn, id)?.ok_or(SalesError::SaleNotFound)
}

pub fn cancel(pool: &DbPool, id: &str, user: &AuthenticatedUser) -> Result<SaleDto, SalesError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn.transaction()?;

    let sale = repo::find_sale(&tx, id)?.ok_or(SalesError::SaleNotFound)?;
    if sale.status != "completed" {
        return Err(SalesError::InvalidStatusTransition(sale.status));
    }

    for item in repo::find_items(&tx, id)? {
        products::adjust_stock(&tx, item.product_id, item.quantity)?;
        stock_movements::insert(
            &tx,
            item.product_id,
            item.quantity,
            "return",
            Some("sale_cancellation"),
            Some(id),
            Some(user.id),
        )?;
    }

    repo::update_status(&tx, id, "cancelled")?;

    audit_logs::insert(
        &tx,
        Some(user.id),
        None,
        "SALE_CANCELLED",
        "sale",
        id,
        Some(r#"{"status":"completed"}"#),
        Some(r#"{"status":"cancelled"}"#),
        None,
    )?;

    tx.commit()?;

    load_sale(&conn, id)?.ok_or(SalesError::SaleNotFound)
}

fn compute_product_discount(
    conn: &Connection,
    discount_id: Option<i64>,
    quantity: i64,
    line_subtotal: i64,
) -> Result<i64, SalesError> {
    let Some(discount_id) = discount_id else {
        return Ok(0);
    };
    let Some(discount) = discounts::find_by_id(conn, discount_id)? else {
        return Ok(0);
    };
    if !discount.is_active {
        return Ok(0);
    }
    let amount = match discount.kind.as_str() {
        "percent" => ((line_subtotal as f64) * (discount.value as f64) / 100.0).round() as i64,
        "fixed" => discount.value * quantity,
        _ => 0,
    };
    Ok(amount.clamp(0, line_subtotal))
}

fn compute_product_tax(
    conn: &Connection,
    tax_id: Option<i64>,
    taxable_amount: i64,
) -> Result<i64, SalesError> {
    let Some(tax_id) = tax_id else {
        return Ok(0);
    };
    let Some(tax) = taxes::find_by_id(conn, tax_id)? else {
        return Ok(0);
    };
    if !tax.is_active {
        return Ok(0);
    }
    Ok(((taxable_amount as f64) * tax.rate).round() as i64)
}

fn load_sale(conn: &Connection, id: &str) -> Result<Option<SaleDto>, SalesError> {
    let Some(sale) = repo::find_sale(conn, id)? else {
        return Ok(None);
    };
    let items = repo::find_items(conn, id)?
        .into_iter()
        .map(|i| SaleItemDto {
            product_id: i.product_id,
            product_name: i.product_name,
            quantity: i.quantity,
            unit_price: i.unit_price,
            discount: i.discount,
            tax: i.tax,
            line_total: i.line_total,
        })
        .collect();
    let payments = repo::find_payments(conn, id)?
        .into_iter()
        .map(|p| PaymentDto {
            method: p.method,
            amount: p.amount,
            received_at: p.received_at,
        })
        .collect();

    Ok(Some(SaleDto {
        id: sale.id,
        invoice_number: sale.invoice_number,
        customer_id: sale.customer_id,
        cashier_name: sale.cashier_name,
        subtotal: sale.subtotal,
        discount_total: sale.discount_total,
        tax_total: sale.tax_total,
        total: sale.total,
        status: sale.status,
        created_at: sale.created_at,
        items,
        payments,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::discounts::{self, DiscountInput};
    use crate::catalog::products::{self, ProductInput};
    use crate::catalog::taxes::{self, TaxInput};
    use crate::catalog::units::{self, UnitInput};
    use crate::db::DbPool;

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-sales-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = crate::db::init(&dir.join("test.sqlite3")).unwrap();
        (pool, dir)
    }

    fn unique() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// A product priced at $10.00 with a 10% tax and a 10% discount, and
    /// the admin user to check out as.
    fn seed(pool: &DbPool, stock: i64) -> (AuthenticatedUser, i64) {
        let user =
            crate::auth::bootstrap_admin(pool, "admin", "supersecret123", "Admin User").unwrap();

        let unit = units::create(
            pool,
            UnitInput {
                name: "Piece".into(),
                abbreviation: "pc".into(),
            },
        )
        .unwrap();
        let tax = taxes::create(
            pool,
            TaxInput {
                name: "VAT".into(),
                rate: 0.10,
            },
        )
        .unwrap();
        let discount = discounts::create(
            pool,
            DiscountInput {
                name: "Promo".into(),
                kind: "percent".into(),
                value: 10,
            },
        )
        .unwrap();

        let product = products::create(
            pool,
            ProductInput {
                sku: "SKU-1".into(),
                name: "Widget".into(),
                description: None,
                category_id: None,
                brand_id: None,
                unit_id: unit.id,
                purchase_price: 500,
                selling_price: 1000,
                tax_id: Some(tax.id),
                discount_id: Some(discount.id),
                min_stock: 0,
                image_path: None,
                barcodes: vec![],
            },
        )
        .unwrap();

        // Products are created with zero stock; set it directly for the test.
        {
            let conn = pool.get().unwrap();
            crate::db::repositories::products::adjust_stock(&conn, product.id, stock).unwrap();
        }

        (user, product.id)
    }

    fn cart(product_id: i64, quantity: i64) -> CheckoutInput {
        CheckoutInput {
            customer_id: None,
            items: vec![CartLineInput {
                product_id,
                quantity,
                discount_override: None,
            }],
            payments: vec![PaymentInput {
                method: "cash".into(),
                amount: 2000,
            }],
            idempotency_key: None,
        }
    }

    #[test]
    fn checkout_computes_tax_and_discount_and_deducts_stock() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 10);

        let result = checkout(&pool, &user, cart(product_id, 2)).unwrap();

        // subtotal 2000, discount 10% = 200, taxable 1800, tax 10% = 180
        assert_eq!(result.sale.subtotal, 2000);
        assert_eq!(result.sale.discount_total, 200);
        assert_eq!(result.sale.tax_total, 180);
        assert_eq!(result.sale.total, 1980);
        assert_eq!(result.change, 20); // paid 2000, total 1980
        assert_eq!(result.sale.items.len(), 1);
        assert_eq!(result.sale.items[0].line_total, 1980);

        let product = products::get(&pool, product_id).unwrap();
        assert_eq!(product.current_stock, 8);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn checkout_rejects_insufficient_stock_with_no_partial_writes() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 1);

        let result = checkout(&pool, &user, cart(product_id, 5));

        assert!(matches!(result, Err(SalesError::InsufficientStock { .. })));

        let product = products::get(&pool, product_id).unwrap();
        assert_eq!(
            product.current_stock, 1,
            "stock must be untouched on failure"
        );
        assert!(list(&pool, SaleListQuery::default()).unwrap().is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn checkout_rejects_when_payments_dont_cover_the_total() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 10);

        let mut input = cart(product_id, 2);
        input.payments = vec![PaymentInput {
            method: "cash".into(),
            amount: 100,
        }];
        let result = checkout(&pool, &user, input);

        assert!(matches!(result, Err(SalesError::Validation(_))));
        let product = products::get(&pool, product_id).unwrap();
        assert_eq!(product.current_stock, 10);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn checkout_supports_mixed_payments() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 10);

        let mut input = cart(product_id, 2);
        input.payments = vec![
            PaymentInput {
                method: "card".into(),
                amount: 1000,
            },
            PaymentInput {
                method: "cash".into(),
                amount: 1000,
            },
        ];
        let result = checkout(&pool, &user, input).unwrap();

        assert_eq!(result.sale.payments.len(), 2);
        assert_eq!(result.change, 20); // paid 2000, total 1980

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn checkout_is_idempotent() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 10);

        let mut input = cart(product_id, 2);
        input.idempotency_key = Some("DEVICE1-SALE-20260101-0001".into());

        let first = checkout(&pool, &user, input).unwrap();
        // Rebuild the input since CheckoutInput isn't Clone; same key, same cart.
        let mut retry = cart(product_id, 2);
        retry.idempotency_key = Some("DEVICE1-SALE-20260101-0001".into());
        let second = checkout(&pool, &user, retry).unwrap();

        assert_eq!(first.sale.id, second.sale.id);
        assert_eq!(list(&pool, SaleListQuery::default()).unwrap().len(), 1);

        let product = products::get(&pool, product_id).unwrap();
        assert_eq!(product.current_stock, 8, "stock must only be deducted once");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cancel_reverses_stock_and_sets_status() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 10);

        let result = checkout(&pool, &user, cart(product_id, 2)).unwrap();
        let cancelled = cancel(&pool, &result.sale.id, &user).unwrap();

        assert_eq!(cancelled.status, "cancelled");
        let product = products::get(&pool, product_id).unwrap();
        assert_eq!(product.current_stock, 10, "stock should be fully restored");

        let result = cancel(&pool, &result.sale.id, &user);
        assert!(matches!(
            result,
            Err(SalesError::InvalidStatusTransition(_))
        ));

        std::fs::remove_dir_all(&dir).ok();
    }
}
