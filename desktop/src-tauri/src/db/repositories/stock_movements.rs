use rusqlite::{params, Connection};

pub struct MovementRow {
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

/// Most recent movements, optionally scoped to one product.
pub fn list(
    conn: &Connection,
    product_id: Option<i64>,
    limit: i64,
) -> rusqlite::Result<Vec<MovementRow>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.product_id, p.name, m.quantity_delta, m.reason,
                m.reference_type, m.reference_id, u.full_name, m.created_at
         FROM stock_movements m
         JOIN products p ON p.id = m.product_id
         LEFT JOIN users u ON u.id = m.user_id
         WHERE ?1 IS NULL OR m.product_id = ?1
         ORDER BY m.created_at DESC, m.id DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![product_id, limit], |row| {
        Ok(MovementRow {
            id: row.get(0)?,
            product_id: row.get(1)?,
            product_name: row.get(2)?,
            quantity_delta: row.get(3)?,
            reason: row.get(4)?,
            reference_type: row.get(5)?,
            reference_id: row.get(6)?,
            user_name: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;
    rows.collect()
}
