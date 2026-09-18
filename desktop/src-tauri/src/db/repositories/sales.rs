use rusqlite::{params, Connection, OptionalExtension};

pub struct SaleRow {
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
}

pub struct SaleItemRow {
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: i64,
    pub discount: i64,
    pub tax: i64,
    pub line_total: i64,
}

pub struct PaymentRow {
    pub method: String,
    pub amount: i64,
    pub received_at: String,
}

#[derive(Default)]
pub struct SaleFilter {
    pub status: Option<String>,
    pub limit: i64,
}

fn map_sale_row(row: &rusqlite::Row) -> rusqlite::Result<SaleRow> {
    Ok(SaleRow {
        id: row.get(0)?,
        invoice_number: row.get(1)?,
        customer_id: row.get(2)?,
        cashier_name: row.get(3)?,
        subtotal: row.get(4)?,
        discount_total: row.get(5)?,
        tax_total: row.get(6)?,
        total: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
    })
}

const SALE_SELECT: &str = "
    s.id, s.invoice_number, s.customer_id, u.full_name,
    s.subtotal, s.discount_total, s.tax_total, s.total, s.status, s.created_at
    FROM sales s
    JOIN users u ON u.id = s.user_id
";

/// Returns the id of an existing sale created with this idempotency key,
/// if any — the caller returns that sale instead of creating a new one.
pub fn find_by_idempotency_key(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT id FROM sales WHERE idempotency_key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

#[allow(clippy::too_many_arguments)]
pub fn insert_sale(
    conn: &Connection,
    id: &str,
    invoice_number: &str,
    customer_id: Option<i64>,
    user_id: i64,
    subtotal: i64,
    discount_total: i64,
    tax_total: i64,
    total: i64,
    idempotency_key: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sales (id, invoice_number, customer_id, user_id, subtotal, discount_total, tax_total, total, idempotency_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, invoice_number, customer_id, user_id, subtotal, discount_total, tax_total, total, idempotency_key],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn insert_sale_item(
    conn: &Connection,
    sale_id: &str,
    product_id: i64,
    quantity: i64,
    unit_price: i64,
    discount: i64,
    tax: i64,
    line_total: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, discount, tax, line_total)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![sale_id, product_id, quantity, unit_price, discount, tax, line_total],
    )?;
    Ok(())
}

pub fn insert_payment(
    conn: &Connection,
    sale_id: &str,
    method: &str,
    amount: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO payments (sale_id, method, amount) VALUES (?1, ?2, ?3)",
        params![sale_id, method, amount],
    )?;
    Ok(())
}

/// `INV-YYYYMMDD-NNNN`, sequential per day. Computed and used inside the
/// same transaction as the sale insert so it stays correct under
/// SQLite's single-writer model.
pub fn next_invoice_number(conn: &Connection) -> rusqlite::Result<String> {
    let today: String = conn.query_row("SELECT strftime('%Y%m%d', 'now')", [], |row| row.get(0))?;
    let count_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales WHERE strftime('%Y%m%d', created_at) = ?1",
        params![today],
        |row| row.get(0),
    )?;
    Ok(format!("INV-{today}-{:04}", count_today + 1))
}

pub fn find_sale(conn: &Connection, id: &str) -> rusqlite::Result<Option<SaleRow>> {
    let sql = format!("SELECT {SALE_SELECT} WHERE s.id = ?1");
    conn.query_row(&sql, params![id], map_sale_row).optional()
}

pub fn list_sales(conn: &Connection, filter: &SaleFilter) -> rusqlite::Result<Vec<SaleRow>> {
    let sql = format!(
        "SELECT {SALE_SELECT}
         WHERE (?1 IS NULL OR s.status = ?1)
         ORDER BY s.created_at DESC
         LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let limit = if filter.limit > 0 { filter.limit } else { 100 };
    let rows = stmt.query_map(params![filter.status, limit], map_sale_row)?;
    rows.collect()
}

pub fn find_items(conn: &Connection, sale_id: &str) -> rusqlite::Result<Vec<SaleItemRow>> {
    let mut stmt = conn.prepare(
        "SELECT si.product_id, p.name, si.quantity, si.unit_price, si.discount, si.tax, si.line_total
         FROM sale_items si
         JOIN products p ON p.id = si.product_id
         WHERE si.sale_id = ?1
         ORDER BY si.id",
    )?;
    let rows = stmt.query_map(params![sale_id], |row| {
        Ok(SaleItemRow {
            product_id: row.get(0)?,
            product_name: row.get(1)?,
            quantity: row.get(2)?,
            unit_price: row.get(3)?,
            discount: row.get(4)?,
            tax: row.get(5)?,
            line_total: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn find_payments(conn: &Connection, sale_id: &str) -> rusqlite::Result<Vec<PaymentRow>> {
    let mut stmt = conn.prepare(
        "SELECT method, amount, received_at FROM payments WHERE sale_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map(params![sale_id], |row| {
        Ok(PaymentRow {
            method: row.get(0)?,
            amount: row.get(1)?,
            received_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn update_status(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE sales SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}
