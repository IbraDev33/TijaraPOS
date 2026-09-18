use serde::{Deserialize, Serialize};

use super::error::{friendly_conflict, CatalogError};
use crate::db::repositories::units as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct UnitDto {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
}

#[derive(Debug, Deserialize)]
pub struct UnitInput {
    pub name: String,
    pub abbreviation: String,
}

impl From<repo::UnitRow> for UnitDto {
    fn from(row: repo::UnitRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            abbreviation: row.abbreviation,
        }
    }
}

fn validate(input: &UnitInput) -> Result<(String, String), CatalogError> {
    let name = input.name.trim();
    let abbreviation = input.abbreviation.trim();
    if name.is_empty() || name.chars().count() > 50 {
        return Err(CatalogError::Validation(
            "Name must be between 1 and 50 characters".into(),
        ));
    }
    if abbreviation.is_empty() || abbreviation.chars().count() > 10 {
        return Err(CatalogError::Validation(
            "Abbreviation must be between 1 and 10 characters".into(),
        ));
    }
    Ok((name.to_string(), abbreviation.to_string()))
}

const CONFLICTS: &[(&str, &str)] = &[(
    "units.abbreviation",
    "A unit with this abbreviation already exists",
)];

pub fn list(pool: &DbPool) -> Result<Vec<UnitDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::list(&conn)?.into_iter().map(UnitDto::from).collect())
}

pub fn create(pool: &DbPool, input: UnitInput) -> Result<UnitDto, CatalogError> {
    let (name, abbreviation) = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    let id =
        repo::insert(&conn, &name, &abbreviation).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    repo::find_by_id(&conn, id)?
        .map(UnitDto::from)
        .ok_or(CatalogError::NotFound("unit"))
}

pub fn update(pool: &DbPool, id: i64, input: UnitInput) -> Result<UnitDto, CatalogError> {
    let (name, abbreviation) = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    repo::update(&conn, id, &name, &abbreviation).map_err(|e| friendly_conflict(e, CONFLICTS))?;
    repo::find_by_id(&conn, id)?
        .map(UnitDto::from)
        .ok_or(CatalogError::NotFound("unit"))
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
        let dir = std::env::temp_dir().join(format!("tijarapos-units-test-{}", unique()));
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
    fn rejects_a_duplicate_abbreviation() {
        let (pool, dir) = test_pool();
        create(
            &pool,
            UnitInput {
                name: "Piece".into(),
                abbreviation: "pc".into(),
            },
        )
        .unwrap();

        let result = create(
            &pool,
            UnitInput {
                name: "Pack".into(),
                abbreviation: "pc".into(),
            },
        );

        assert!(matches!(result, Err(CatalogError::Conflict(_))));
        std::fs::remove_dir_all(&dir).ok();
    }
}
