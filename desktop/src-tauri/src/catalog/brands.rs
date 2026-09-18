use serde::{Deserialize, Serialize};

use super::error::{friendly_conflict, CatalogError};
use crate::db::repositories::brands as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct BrandDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct BrandInput {
    pub name: String,
}

impl From<repo::BrandRow> for BrandDto {
    fn from(row: repo::BrandRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
        }
    }
}

fn validate_name(name: &str) -> Result<String, CatalogError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err(CatalogError::Validation(
            "Name must be between 1 and 100 characters".into(),
        ));
    }
    Ok(trimmed.to_string())
}

const CONFLICTS: &[(&str, &str)] = &[("brands.name", "A brand with this name already exists")];

pub fn list(pool: &DbPool) -> Result<Vec<BrandDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::list(&conn)?.into_iter().map(BrandDto::from).collect())
}

pub fn create(pool: &DbPool, input: BrandInput) -> Result<BrandDto, CatalogError> {
    let name = validate_name(&input.name)?;
    let conn = pool.get().map_err(DbError::from)?;
    let id = repo::insert(&conn, &name).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    repo::find_by_id(&conn, id)?
        .map(BrandDto::from)
        .ok_or(CatalogError::NotFound("brand"))
}

pub fn update(pool: &DbPool, id: i64, input: BrandInput) -> Result<BrandDto, CatalogError> {
    let name = validate_name(&input.name)?;
    let conn = pool.get().map_err(DbError::from)?;
    repo::update(&conn, id, &name).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    repo::find_by_id(&conn, id)?
        .map(BrandDto::from)
        .ok_or(CatalogError::NotFound("brand"))
}

pub fn delete(pool: &DbPool, id: i64) -> Result<(), CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::soft_delete(&conn, id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-brands-test-{}", unique()));
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

    #[test]
    fn rejects_a_duplicate_brand_name() {
        let (pool, dir) = test_pool();
        create(
            &pool,
            BrandInput {
                name: "Acme".into(),
            },
        )
        .unwrap();

        let result = create(
            &pool,
            BrandInput {
                name: "Acme".into(),
            },
        );

        assert!(matches!(result, Err(CatalogError::Conflict(_))));
        std::fs::remove_dir_all(&dir).ok();
    }
}
