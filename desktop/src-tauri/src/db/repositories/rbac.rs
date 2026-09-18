use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

pub fn role_id_by_name(conn: &Connection, name: &str) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM roles WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )
    .optional()
}

pub fn assign_role(conn: &Connection, user_id: i64, role_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
        params![user_id, role_id],
    )?;
    Ok(())
}

/// The union of permission keys across every role a user holds.
pub fn permissions_for_user(conn: &Connection, user_id: i64) -> rusqlite::Result<HashSet<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT p.key
         FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN user_roles ur ON ur.role_id = rp.role_id
         WHERE ur.user_id = ?1",
    )?;
    let rows = stmt.query_map(params![user_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}
