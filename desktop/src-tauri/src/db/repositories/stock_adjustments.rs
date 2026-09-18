use rusqlite::{params, Connection};

/// The human-facing record behind a manual stock correction (see
/// docs/database.md). Each one produces exactly one `stock_movements`
/// row, inserted separately by the caller in the same transaction.
pub fn insert(
    conn: &Connection,
    product_id: i64,
    quantity_delta: i64,
    reason: &str,
    note: Option<&str>,
    user_id: i64,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO stock_adjustments (product_id, quantity_delta, reason, note, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![product_id, quantity_delta, reason, note, user_id],
    )?;
    Ok(conn.last_insert_rowid())
}
