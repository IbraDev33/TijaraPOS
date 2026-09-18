use rusqlite::{params, Connection, OptionalExtension};

pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub full_name: String,
    pub is_active: bool,
}

pub fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
}

pub fn find_by_username(conn: &Connection, username: &str) -> rusqlite::Result<Option<UserRow>> {
    conn.query_row(
        "SELECT id, username, password_hash, full_name, is_active FROM users WHERE username = ?1",
        params![username],
        |row| {
            Ok(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                full_name: row.get(3)?,
                is_active: row.get::<_, i64>(4)? != 0,
            })
        },
    )
    .optional()
}

pub fn insert(
    conn: &Connection,
    username: &str,
    password_hash: &str,
    full_name: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO users (username, password_hash, full_name) VALUES (?1, ?2, ?3)",
        params![username, password_hash, full_name],
    )?;
    Ok(conn.last_insert_rowid())
}
