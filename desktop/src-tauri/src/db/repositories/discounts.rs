use rusqlite::{params, Connection, OptionalExtension};

pub struct DiscountRow {
    pub id: i64,
    pub name: String,
    pub kind: String, // "percent" | "fixed"
    pub value: i64,
    pub is_active: bool,
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<DiscountRow> {
    Ok(DiscountRow {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        value: row.get(3)?,
        is_active: row.get::<_, i64>(4)? != 0,
    })
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<DiscountRow>> {
    let mut stmt =
        conn.prepare("SELECT id, name, type, value, is_active FROM discounts ORDER BY name")?;
    let rows = stmt.query_map([], map_row)?;
    rows.collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<DiscountRow>> {
    conn.query_row(
        "SELECT id, name, type, value, is_active FROM discounts WHERE id = ?1",
        params![id],
        map_row,
    )
    .optional()
}

pub fn insert(conn: &Connection, name: &str, kind: &str, value: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO discounts (name, type, value) VALUES (?1, ?2, ?3)",
        params![name, kind, value],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    kind: &str,
    value: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE discounts SET name = ?1, type = ?2, value = ?3 WHERE id = ?4",
        params![name, kind, value, id],
    )?;
    Ok(())
}

pub fn set_active(conn: &Connection, id: i64, is_active: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE discounts SET is_active = ?1 WHERE id = ?2",
        params![is_active as i64, id],
    )?;
    Ok(())
}
