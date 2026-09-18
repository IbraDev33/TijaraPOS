use rusqlite::{params, Connection, OptionalExtension};

pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub is_active: bool,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CategoryRow> {
    Ok(CategoryRow {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        is_active: row.get::<_, i64>(3)? != 0,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<CategoryRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, is_active FROM categories WHERE deleted_at IS NULL ORDER BY name",
    )?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<CategoryRow>> {
    conn.query_row(
        "SELECT id, name, parent_id, is_active FROM categories WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_row,
    )
    .optional()
}

pub fn insert(conn: &Connection, name: &str, parent_id: Option<i64>) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
        params![name, parent_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    parent_id: Option<i64>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE categories SET name = ?1, parent_id = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![name, parent_id, id],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE categories
         SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), is_active = 0
         WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
