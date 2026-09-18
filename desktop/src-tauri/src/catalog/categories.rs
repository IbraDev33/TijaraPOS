use serde::{Deserialize, Serialize};

use super::error::CatalogError;
use crate::db::repositories::categories as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct CategoryDto {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub name: String,
    pub parent_id: Option<i64>,
}

impl From<repo::CategoryRow> for CategoryDto {
    fn from(row: repo::CategoryRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            parent_id: row.parent_id,
            is_active: row.is_active,
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

pub fn list(pool: &DbPool) -> Result<Vec<CategoryDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::list(&conn)?
        .into_iter()
        .map(CategoryDto::from)
        .collect())
}

pub fn create(pool: &DbPool, input: CategoryInput) -> Result<CategoryDto, CatalogError> {
    let name = validate_name(&input.name)?;
    let conn = pool.get().map_err(DbError::from)?;
    let id = repo::insert(&conn, &name, input.parent_id)?;
    repo::find_by_id(&conn, id)?
        .map(CategoryDto::from)
        .ok_or(CatalogError::NotFound("category"))
}

pub fn update(pool: &DbPool, id: i64, input: CategoryInput) -> Result<CategoryDto, CatalogError> {
    let name = validate_name(&input.name)?;
    if input.parent_id == Some(id) {
        return Err(CatalogError::Validation(
            "A category cannot be its own parent".into(),
        ));
    }
    let conn = pool.get().map_err(DbError::from)?;
    repo::update(&conn, id, &name, input.parent_id)?;
    repo::find_by_id(&conn, id)?
        .map(CategoryDto::from)
        .ok_or(CatalogError::NotFound("category"))
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
        let dir = std::env::temp_dir().join(format!("tijarapos-categories-test-{}", unique()));
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
    fn creates_updates_and_soft_deletes_a_category() {
        let (pool, dir) = test_pool();

        let created = create(
            &pool,
            CategoryInput {
                name: "  Beverages  ".into(),
                parent_id: None,
            },
        )
        .unwrap();
        assert_eq!(created.name, "Beverages");

        let updated = update(
            &pool,
            created.id,
            CategoryInput {
                name: "Drinks".into(),
                parent_id: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "Drinks");

        delete(&pool, created.id).unwrap();
        assert!(list(&pool).unwrap().iter().all(|c| c.id != created.id));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_a_category_being_its_own_parent() {
        let (pool, dir) = test_pool();
        let created = create(
            &pool,
            CategoryInput {
                name: "Snacks".into(),
                parent_id: None,
            },
        )
        .unwrap();

        let result = update(
            &pool,
            created.id,
            CategoryInput {
                name: "Snacks".into(),
                parent_id: Some(created.id),
            },
        );

        assert!(matches!(result, Err(CatalogError::Validation(_))));
        std::fs::remove_dir_all(&dir).ok();
    }
}
