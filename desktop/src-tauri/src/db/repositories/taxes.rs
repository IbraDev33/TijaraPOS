use rusqlite::{params, Connection, OptionalExtension};

pub struct TaxRow {
    pub id: i64,
    pub name: String,
    pub rate: f64,
    pub is_active: bool,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<TaxRow> {
    Ok(TaxRow {
        id: row.get(0)?,
        name: row.get(1)?,
        rate: row.get(2)?,
        is_active: row.get::<_, i64>(3)? != 0,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<TaxRow>> {
    let mut stmt = conn.prepare("SELECT id, name, rate, is_active FROM taxes ORDER BY name")?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<TaxRow>> {
    conn.query_row(
        "SELECT id, name, rate, is_active FROM taxes WHERE id = ?1",
        params![id],
        map_row,
    )
    .optional()
}

pub fn insert(conn: &Connection, name: &str, rate: f64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO taxes (name, rate) VALUES (?1, ?2)",
        params![name, rate],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, name: &str, rate: f64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE taxes SET name = ?1, rate = ?2 WHERE id = ?3",
        params![name, rate, id],
    )?;
    Ok(())
}

pub fn set_active(conn: &Connection, id: i64, is_active: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE taxes SET is_active = ?1 WHERE id = ?2",
        params![is_active as i64, id],
    )?;
    Ok(())
}
