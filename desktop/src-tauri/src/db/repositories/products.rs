use rusqlite::{params, Connection, OptionalExtension};

pub struct ProductRow {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub brand_id: Option<i64>,
    pub brand_name: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub unit_abbreviation: String,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub tax_id: Option<i64>,
    pub discount_id: Option<i64>,
    pub min_stock: i64,
    pub current_stock: i64,
    pub is_active: bool,
    pub image_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub barcodes: Vec<String>,
}

#[derive(Default)]
pub struct ProductFilter {
    /// Already wrapped with `%...%` by the caller.
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub include_inactive: bool,
}

pub struct NewProduct<'a> {
    pub sku: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub category_id: Option<i64>,
    pub brand_id: Option<i64>,
    pub unit_id: i64,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub tax_id: Option<i64>,
    pub discount_id: Option<i64>,
    pub min_stock: i64,
    pub image_path: Option<&'a str>,
    pub barcodes: &'a [String],
}

const SELECT_COLUMNS: &str = "
    p.id, p.sku, p.name, p.description,
    p.category_id, c.name,
    p.brand_id, b.name,
    p.unit_id, u.name, u.abbreviation,
    p.purchase_price, p.selling_price,
    p.tax_id, p.discount_id,
    p.min_stock, p.current_stock, p.is_active, p.image_path,
    p.created_at, p.updated_at
";

const FROM_CLAUSE: &str = "
    FROM products p
    JOIN units u ON u.id = p.unit_id
    LEFT JOIN categories c ON c.id = p.category_id
    LEFT JOIN brands b ON b.id = p.brand_id
";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ProductRow> {
    Ok(ProductRow {
        id: row.get(0)?,
        sku: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        category_id: row.get(4)?,
        category_name: row.get(5)?,
        brand_id: row.get(6)?,
        brand_name: row.get(7)?,
        unit_id: row.get(8)?,
        unit_name: row.get(9)?,
        unit_abbreviation: row.get(10)?,
        purchase_price: row.get(11)?,
        selling_price: row.get(12)?,
        tax_id: row.get(13)?,
        discount_id: row.get(14)?,
        min_stock: row.get(15)?,
        current_stock: row.get(16)?,
        is_active: row.get::<_, i64>(17)? != 0,
        image_path: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
        barcodes: Vec::new(),
    })
}

pub fn barcodes_for(conn: &Connection, product_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT barcode FROM product_barcodes WHERE product_id = ?1 ORDER BY barcode")?;
    let rows = stmt.query_map(params![product_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

fn with_barcodes(conn: &Connection, mut product: ProductRow) -> rusqlite::Result<ProductRow> {
    product.barcodes = barcodes_for(conn, product.id)?;
    Ok(product)
}

pub fn list(conn: &Connection, filter: &ProductFilter) -> rusqlite::Result<Vec<ProductRow>> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS} {FROM_CLAUSE}
         WHERE p.deleted_at IS NULL
           AND (?1 = 1 OR p.is_active = 1)
           AND (?2 IS NULL OR p.category_id = ?2)
           AND (?3 IS NULL OR p.sku LIKE ?3 OR p.name LIKE ?3)
         ORDER BY p.name"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<ProductRow> = stmt
        .query_map(
            params![
                filter.include_inactive as i64,
                filter.category_id,
                filter.search
            ],
            map_row,
        )?
        .collect::<rusqlite::Result<_>>()?;

    rows.into_iter()
        .map(|row| with_barcodes(conn, row))
        .collect()
}

pub fn find_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<ProductRow>> {
    let sql =
        format!("SELECT {SELECT_COLUMNS} {FROM_CLAUSE} WHERE p.id = ?1 AND p.deleted_at IS NULL");
    let product = conn.query_row(&sql, params![id], map_row).optional()?;
    product.map(|p| with_barcodes(conn, p)).transpose()
}

pub fn find_by_barcode(conn: &Connection, barcode: &str) -> rusqlite::Result<Option<ProductRow>> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS} {FROM_CLAUSE}
         JOIN product_barcodes pb ON pb.product_id = p.id
         WHERE pb.barcode = ?1 AND p.deleted_at IS NULL"
    );
    let product = conn.query_row(&sql, params![barcode], map_row).optional()?;
    product.map(|p| with_barcodes(conn, p)).transpose()
}

fn replace_barcodes(
    conn: &Connection,
    product_id: i64,
    barcodes: &[String],
) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM product_barcodes WHERE product_id = ?1",
        params![product_id],
    )?;
    for barcode in barcodes {
        conn.execute(
            "INSERT INTO product_barcodes (product_id, barcode) VALUES (?1, ?2)",
            params![product_id, barcode],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, input: &NewProduct) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO products (
            sku, name, description, category_id, brand_id, unit_id,
            purchase_price, selling_price, tax_id, discount_id, min_stock, image_path
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            input.sku,
            input.name,
            input.description,
            input.category_id,
            input.brand_id,
            input.unit_id,
            input.purchase_price,
            input.selling_price,
            input.tax_id,
            input.discount_id,
            input.min_stock,
            input.image_path,
        ],
    )?;
    let product_id = conn.last_insert_rowid();
    replace_barcodes(conn, product_id, input.barcodes)?;
    Ok(product_id)
}

pub fn update(conn: &Connection, id: i64, input: &NewProduct) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE products SET
            sku = ?1, name = ?2, description = ?3, category_id = ?4, brand_id = ?5,
            unit_id = ?6, purchase_price = ?7, selling_price = ?8, tax_id = ?9,
            discount_id = ?10, min_stock = ?11, image_path = ?12
         WHERE id = ?13 AND deleted_at IS NULL",
        params![
            input.sku,
            input.name,
            input.description,
            input.category_id,
            input.brand_id,
            input.unit_id,
            input.purchase_price,
            input.selling_price,
            input.tax_id,
            input.discount_id,
            input.min_stock,
            input.image_path,
            id,
        ],
    )?;
    replace_barcodes(conn, id, input.barcodes)?;
    Ok(())
}

pub fn set_active(conn: &Connection, id: i64, is_active: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE products SET is_active = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![is_active as i64, id],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE products
         SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), is_active = 0
         WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
