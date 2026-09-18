use rusqlite::{params, Connection, OptionalExtension};

pub struct BrandRow {
    pub id: i64,
    pub name: String,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<BrandRow> {
    Ok(BrandRow {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<BrandRow>> {
    let mut stmt =
        conn.prepare("SELECT id, name FROM brands WHERE deleted_at IS NULL ORDER BY name")?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<BrandRow>> {
    conn.query_row(
        "SELECT id, name FROM brands WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_row,
    )
    .optional()
}

pub fn insert(conn: &Connection, name: &str) -> rusqlite::Result<i64> {
    conn.execute("INSERT INTO brands (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE brands SET name = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![name, id],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE brands SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
