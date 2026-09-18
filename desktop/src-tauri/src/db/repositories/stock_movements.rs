use rusqlite::{params, Connection};

/// Append-only. `quantity_delta` is negative for stock leaving (a sale)
/// and positive for stock arriving (a purchase, a cancelled sale's
/// reversal, a positive adjustment).
pub fn insert(
    conn: &Connection,
    product_id: i64,
    quantity_delta: i64,
    reason: &str,
    reference_type: Option<&str>,
    reference_id: Option<&str>,
    user_id: Option<i64>,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO stock_movements (product_id, quantity_delta, reason, reference_type, reference_id, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![product_id, quantity_delta, reason, reference_type, reference_id, user_id],
    )?;
    Ok(conn.last_insert_rowid())
}
