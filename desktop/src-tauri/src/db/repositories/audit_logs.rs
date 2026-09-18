use rusqlite::{params, Connection};

/// Append-only. No role is ever granted delete access to this table —
/// there is deliberately no `delete` function here.
#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    user_id: Option<i64>,
    device_id: Option<i64>,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    before: Option<&str>,
    after: Option<&str>,
    ip_address: Option<&str>,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO audit_logs (user_id, device_id, action, entity_type, entity_id, before, after, ip_address)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![user_id, device_id, action, entity_type, entity_id, before, after, ip_address],
    )?;
    Ok(conn.last_insert_rowid())
}
