//! Manual stock corrections and inventory visibility. Sales' own stock
//! deduction lives in `sales::checkout`; this module is everything else
//! that changes `current_stock` — a human correcting a count, marking
//! damaged goods, recording a return, or moving stock elsewhere. Every
//! change here still goes through `stock_movements`, same as a sale.

pub mod error;
pub use error::InventoryError;

use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::db::repositories::{audit_logs, products, stock_adjustments, stock_movements};
use crate::db::{DbError, DbPool};

const VALID_REASONS: &[&str] = &["adjustment", "return", "damage", "transfer"];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjustmentInput {
    pub product_id: i64,
    /// Positive to add stock, negative to remove it.
    pub quantity_delta: i64,
    pub reason: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjustmentResult {
    pub product_id: i64,
    pub previous_stock: i64,
    pub current_stock: i64,
}

#[derive(Debug, Serialize)]
pub struct MovementDto {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity_delta: i64,
    pub reason: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub user_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MovementListQuery {
    pub product_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LowStockProductDto {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub current_stock: i64,
    pub min_stock: i64,
}

pub fn adjust_stock(
    pool: &DbPool,
    user: &AuthenticatedUser,
    input: StockAdjustmentInput,
) -> Result<StockAdjustmentResult, InventoryError> {
    if input.quantity_delta == 0 {
        return Err(InventoryError::Validation(
            "Quantity change must not be zero".into(),
        ));
    }
    if !VALID_REASONS.contains(&input.reason.as_str()) {
        return Err(InventoryError::Validation(format!(
            "Reason must be one of: {}",
            VALID_REASONS.join(", ")
        )));
    }
    let note = input
        .note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn.transaction()?;

    let product =
        products::find_by_id(&tx, input.product_id)?.ok_or(InventoryError::ProductNotFound)?;
    let new_stock = product.current_stock + input.quantity_delta;
    if new_stock < 0 {
        return Err(InventoryError::WouldGoNegative {
            product_name: product.name,
            current_stock: product.current_stock,
            requested_delta: input.quantity_delta,
        });
    }

    let adjustment_id = stock_adjustments::insert(
        &tx,
        input.product_id,
        input.quantity_delta,
        &input.reason,
        note,
        user.id,
    )?;
    stock_movements::insert(
        &tx,
        input.product_id,
        input.quantity_delta,
        &input.reason,
        Some("stock_adjustment"),
        Some(&adjustment_id.to_string()),
        Some(user.id),
    )?;
    products::adjust_stock(&tx, input.product_id, input.quantity_delta)?;

    audit_logs::insert(
        &tx,
        Some(user.id),
        None,
        "STOCK_ADJUSTED",
        "product",
        &input.product_id.to_string(),
        Some(&format!(r#"{{"current_stock":{}}}"#, product.current_stock)),
        Some(&format!(r#"{{"current_stock":{new_stock}}}"#)),
        None,
    )?;

    tx.commit()?;

    Ok(StockAdjustmentResult {
        product_id: input.product_id,
        previous_stock: product.current_stock,
        current_stock: new_stock,
    })
}

pub fn list_movements(
    pool: &DbPool,
    query: MovementListQuery,
) -> Result<Vec<MovementDto>, InventoryError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(stock_movements::list(&conn, query.product_id, 200)?
        .into_iter()
        .map(|m| MovementDto {
            id: m.id,
            product_id: m.product_id,
            product_name: m.product_name,
            quantity_delta: m.quantity_delta,
            reason: m.reason,
            reference_type: m.reference_type,
            reference_id: m.reference_id,
            user_name: m.user_name,
            created_at: m.created_at,
        })
        .collect())
}

pub fn low_stock(pool: &DbPool) -> Result<Vec<LowStockProductDto>, InventoryError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(products::list_low_stock(&conn)?
        .into_iter()
        .map(|p| LowStockProductDto {
            id: p.id,
            sku: p.sku,
            name: p.name,
            current_stock: p.current_stock,
            min_stock: p.min_stock,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::products::{self as catalog_products, ProductInput};
    use crate::catalog::units::{self, UnitInput};

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-inventory-test-{}", unique()));
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

    fn seed(pool: &DbPool, min_stock: i64) -> (AuthenticatedUser, i64) {
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
        let product = catalog_products::create(
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
                tax_id: None,
                discount_id: None,
                min_stock,
                image_path: None,
                barcodes: vec![],
            },
        )
        .unwrap();
        (user, product.id)
    }

    #[test]
    fn adjust_stock_increases_and_decreases_correctly() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 0);

        let result = adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id,
                quantity_delta: 10,
                reason: "adjustment".into(),
                note: Some("Initial count".into()),
            },
        )
        .unwrap();
        assert_eq!(result.current_stock, 10);

        let result = adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id,
                quantity_delta: -3,
                reason: "damage".into(),
                note: None,
            },
        )
        .unwrap();
        assert_eq!(result.current_stock, 7);

        let product = catalog_products::get(&pool, product_id).unwrap();
        assert_eq!(product.current_stock, 7);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn adjust_stock_rejects_a_delta_that_would_go_negative() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 0);

        let result = adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id,
                quantity_delta: -1,
                reason: "damage".into(),
                note: None,
            },
        );

        assert!(matches!(
            result,
            Err(InventoryError::WouldGoNegative { .. })
        ));
        let product = catalog_products::get(&pool, product_id).unwrap();
        assert_eq!(
            product.current_stock, 0,
            "stock must be untouched on failure"
        );
        assert!(list_movements(&pool, MovementListQuery::default())
            .unwrap()
            .is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn adjust_stock_rejects_an_unknown_reason() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 0);

        let result = adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id,
                quantity_delta: 5,
                reason: "sale".into(),
                note: None,
            },
        );

        assert!(matches!(result, Err(InventoryError::Validation(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn low_stock_returns_only_products_at_or_below_their_minimum() {
        let (pool, dir) = test_pool();
        let (user, low_product) = seed(&pool, 5);
        adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id: low_product,
                quantity_delta: 3,
                reason: "adjustment".into(),
                note: None,
            },
        )
        .unwrap();

        let unit = units::create(
            &pool,
            UnitInput {
                name: "Box".into(),
                abbreviation: "bx".into(),
            },
        )
        .unwrap();
        let well_stocked = catalog_products::create(
            &pool,
            ProductInput {
                sku: "SKU-2".into(),
                name: "Gadget".into(),
                description: None,
                category_id: None,
                brand_id: None,
                unit_id: unit.id,
                purchase_price: 500,
                selling_price: 1000,
                tax_id: None,
                discount_id: None,
                min_stock: 5,
                image_path: None,
                barcodes: vec![],
            },
        )
        .unwrap();
        adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id: well_stocked.id,
                quantity_delta: 50,
                reason: "adjustment".into(),
                note: None,
            },
        )
        .unwrap();

        let low = low_stock(&pool).unwrap();
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].id, low_product);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_movements_can_be_scoped_to_one_product() {
        let (pool, dir) = test_pool();
        let (user, product_id) = seed(&pool, 0);
        adjust_stock(
            &pool,
            &user,
            StockAdjustmentInput {
                product_id,
                quantity_delta: 10,
                reason: "adjustment".into(),
                note: None,
            },
        )
        .unwrap();

        let movements = list_movements(
            &pool,
            MovementListQuery {
                product_id: Some(product_id),
            },
        )
        .unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].quantity_delta, 10);
        assert_eq!(movements[0].reason, "adjustment");

        std::fs::remove_dir_all(&dir).ok();
    }
}
