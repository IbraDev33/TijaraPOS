use rusqlite::{params, Connection, OptionalExtension};

pub struct UnitRow {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<UnitRow> {
    Ok(UnitRow {
        id: row.get(0)?,
        name: row.get(1)?,
        abbreviation: row.get(2)?,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<UnitRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, abbreviation FROM units WHERE deleted_at IS NULL ORDER BY name",
    )?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<UnitRow>> {
    conn.query_row(
        "SELECT id, name, abbreviation FROM units WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        map_row,
    )
    .optional()
}

pub fn insert(conn: &Connection, name: &str, abbreviation: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO units (name, abbreviation) VALUES (?1, ?2)",
        params![name, abbreviation],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, name: &str, abbreviation: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE units SET name = ?1, abbreviation = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![name, abbreviation, id],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE units SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
