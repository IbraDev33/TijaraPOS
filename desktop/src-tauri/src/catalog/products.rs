use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::error::{friendly_conflict, CatalogError};
use crate::db::repositories::products as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct ProductDto {
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
    pub barcodes: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<repo::ProductRow> for ProductDto {
    fn from(row: repo::ProductRow) -> Self {
        Self {
            id: row.id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            category_id: row.category_id,
            category_name: row.category_name,
            brand_id: row.brand_id,
            brand_name: row.brand_name,
            unit_id: row.unit_id,
            unit_name: row.unit_name,
            unit_abbreviation: row.unit_abbreviation,
            purchase_price: row.purchase_price,
            selling_price: row.selling_price,
            tax_id: row.tax_id,
            discount_id: row.discount_id,
            min_stock: row.min_stock,
            current_stock: row.current_stock,
            is_active: row.is_active,
            image_path: row.image_path,
            barcodes: row.barcodes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInput {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub brand_id: Option<i64>,
    pub unit_id: i64,
    pub purchase_price: i64,
    pub selling_price: i64,
    pub tax_id: Option<i64>,
    pub discount_id: Option<i64>,
    pub min_stock: i64,
    pub image_path: Option<String>,
    pub barcodes: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProductListQuery {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    #[serde(default)]
    pub include_inactive: bool,
}

struct ValidatedProduct {
    sku: String,
    name: String,
    description: Option<String>,
    barcodes: Vec<String>,
}

fn validate(input: &ProductInput) -> Result<ValidatedProduct, CatalogError> {
    let sku = input.sku.trim();
    if sku.is_empty() || sku.chars().count() > 64 {
        return Err(CatalogError::Validation(
            "SKU must be between 1 and 64 characters".into(),
        ));
    }
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 200 {
        return Err(CatalogError::Validation(
            "Name must be between 1 and 200 characters".into(),
        ));
    }
    if input.purchase_price < 0 || input.selling_price < 0 {
        return Err(CatalogError::Validation(
            "Prices must not be negative".into(),
        ));
    }
    if input.min_stock < 0 {
        return Err(CatalogError::Validation(
            "Minimum stock must not be negative".into(),
        ));
    }

    let description = input
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let mut seen = HashSet::new();
    let mut barcodes = Vec::new();
    for barcode in &input.barcodes {
        let trimmed = barcode.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > 64 {
            return Err(CatalogError::Validation(
                "Barcodes must be at most 64 characters".into(),
            ));
        }
        if !seen.insert(trimmed.to_string()) {
            return Err(CatalogError::Validation(format!(
                "Duplicate barcode: {trimmed}"
            )));
        }
        barcodes.push(trimmed.to_string());
    }

    Ok(ValidatedProduct {
        sku: sku.to_string(),
        name: name.to_string(),
        description,
        barcodes,
    })
}

const CONFLICTS: &[(&str, &str)] = &[
    ("products.sku", "A product with this SKU already exists"),
    (
        "product_barcodes.barcode",
        "One of these barcodes is already assigned to another product",
    ),
];

pub fn list(pool: &DbPool, query: ProductListQuery) -> Result<Vec<ProductDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    let filter = repo::ProductFilter {
        search: query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{s}%")),
        category_id: query.category_id,
        include_inactive: query.include_inactive,
    };
    Ok(repo::list(&conn, &filter)?
        .into_iter()
        .map(ProductDto::from)
        .collect())
}

pub fn get(pool: &DbPool, id: i64) -> Result<ProductDto, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::find_by_id(&conn, id)?
        .map(ProductDto::from)
        .ok_or(CatalogError::NotFound("product"))
}

pub fn get_by_barcode(pool: &DbPool, barcode: &str) -> Result<Option<ProductDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::find_by_barcode(&conn, barcode.trim())?.map(ProductDto::from))
}

pub fn create(pool: &DbPool, input: ProductInput) -> Result<ProductDto, CatalogError> {
    let validated = validate(&input)?;
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn.transaction()?;

    let new_product = repo::NewProduct {
        sku: &validated.sku,
        name: &validated.name,
        description: validated.description.as_deref(),
        category_id: input.category_id,
        brand_id: input.brand_id,
        unit_id: input.unit_id,
        purchase_price: input.purchase_price,
        selling_price: input.selling_price,
        tax_id: input.tax_id,
        discount_id: input.discount_id,
        min_stock: input.min_stock,
        image_path: input.image_path.as_deref(),
        barcodes: &validated.barcodes,
    };
    let id = repo::insert(&tx, &new_product).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    tx.commit()?;

    repo::find_by_id(&conn, id)?
        .map(ProductDto::from)
        .ok_or(CatalogError::NotFound("product"))
}

pub fn update(pool: &DbPool, id: i64, input: ProductInput) -> Result<ProductDto, CatalogError> {
    let validated = validate(&input)?;
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn.transaction()?;

    let updated_product = repo::NewProduct {
        sku: &validated.sku,
        name: &validated.name,
        description: validated.description.as_deref(),
        category_id: input.category_id,
        brand_id: input.brand_id,
        unit_id: input.unit_id,
        purchase_price: input.purchase_price,
        selling_price: input.selling_price,
        tax_id: input.tax_id,
        discount_id: input.discount_id,
        min_stock: input.min_stock,
        image_path: input.image_path.as_deref(),
        barcodes: &validated.barcodes,
    };
    repo::update(&tx, id, &updated_product).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    tx.commit()?;

    repo::find_by_id(&conn, id)?
        .map(ProductDto::from)
        .ok_or(CatalogError::NotFound("product"))
}

pub fn set_active(pool: &DbPool, id: i64, is_active: bool) -> Result<(), CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::set_active(&conn, id, is_active)?;
    Ok(())
}

pub fn delete(pool: &DbPool, id: i64) -> Result<(), CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::soft_delete(&conn, id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::units::{self, UnitInput};
    use crate::db::DbPool;

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-products-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = crate::db::init(&dir.join("test.sqlite3")).unwrap();
        (pool, dir)
    }

    fn unique() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    fn seed_unit(pool: &DbPool) -> i64 {
        units::create(
            pool,
            UnitInput {
                name: "Piece".into(),
                abbreviation: "pc".into(),
            },
        )
        .unwrap()
        .id
    }

    fn sample_input(sku: &str, unit_id: i64, barcodes: Vec<String>) -> ProductInput {
        ProductInput {
            sku: sku.to_string(),
            name: "Test product".to_string(),
            description: Some("  a description  ".to_string()),
            category_id: None,
            brand_id: None,
            unit_id,
            purchase_price: 100,
            selling_price: 200,
            tax_id: None,
            discount_id: None,
            min_stock: 5,
            image_path: None,
            barcodes,
        }
    }

    #[test]
    fn creates_a_product_with_barcodes_and_trims_description() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);

        let product = create(
            &pool,
            sample_input("SKU-1", unit_id, vec!["111".into(), "222".into()]),
        )
        .unwrap();

        assert_eq!(product.sku, "SKU-1");
        assert_eq!(product.description.as_deref(), Some("a description"));
        assert_eq!(product.current_stock, 0);
        assert_eq!(product.barcodes, vec!["111".to_string(), "222".to_string()]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_duplicate_sku_with_a_friendly_conflict() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        create(&pool, sample_input("SKU-1", unit_id, vec![])).unwrap();

        let result = create(&pool, sample_input("SKU-1", unit_id, vec![]));

        assert!(matches!(result, Err(CatalogError::Conflict(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_a_barcode_already_used_by_another_product() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        create(&pool, sample_input("SKU-1", unit_id, vec!["999".into()])).unwrap();

        let result = create(&pool, sample_input("SKU-2", unit_id, vec!["999".into()]));

        assert!(matches!(result, Err(CatalogError::Conflict(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn update_replaces_the_barcode_set() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        let product = create(&pool, sample_input("SKU-1", unit_id, vec!["111".into()])).unwrap();

        let mut input = sample_input("SKU-1", unit_id, vec!["222".into(), "333".into()]);
        input.name = "Renamed".into();
        let updated = update(&pool, product.id, input).unwrap();

        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.barcodes, vec!["222".to_string(), "333".to_string()]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_by_barcode_locates_the_owning_product() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        let product = create(&pool, sample_input("SKU-1", unit_id, vec!["555".into()])).unwrap();

        let found = get_by_barcode(&pool, "555").unwrap();

        assert_eq!(found.map(|p| p.id), Some(product.id));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn soft_deleted_products_are_excluded_from_list_and_lookup() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        let product = create(&pool, sample_input("SKU-1", unit_id, vec![])).unwrap();

        delete(&pool, product.id).unwrap();

        assert!(get(&pool, product.id).is_err());
        assert!(list(&pool, ProductListQuery::default()).unwrap().is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_negative_prices() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);

        let mut input = sample_input("SKU-1", unit_id, vec![]);
        input.selling_price = -1;
        let result = create(&pool, input);

        assert!(matches!(result, Err(CatalogError::Validation(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn search_filters_by_name_or_sku() {
        let (pool, dir) = test_pool();
        let unit_id = seed_unit(&pool);
        create(&pool, sample_input("APPLE-1", unit_id, vec![])).unwrap();
        let mut other = sample_input("BANANA-1", unit_id, vec![]);
        other.name = "Banana".into();
        create(&pool, other).unwrap();

        let results = list(
            &pool,
            ProductListQuery {
                search: Some("banana".into()),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Banana");
        std::fs::remove_dir_all(&dir).ok();
    }
}
