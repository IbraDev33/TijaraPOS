use serde::{Deserialize, Serialize};

use super::error::CatalogError;
use crate::db::repositories::taxes as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct TaxDto {
    pub id: i64,
    pub name: String,
    pub rate: f64,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct TaxInput {
    pub name: String,
    pub rate: f64,
}

impl From<repo::TaxRow> for TaxDto {
    fn from(row: repo::TaxRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            rate: row.rate,
            is_active: row.is_active,
        }
    }
}

fn validate(input: &TaxInput) -> Result<String, CatalogError> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(CatalogError::Validation(
            "Name must be between 1 and 100 characters".into(),
        ));
    }
    if !(0.0..=1.0).contains(&input.rate) {
        return Err(CatalogError::Validation(
            "Rate must be between 0 and 1 (e.g. 0.2 for 20%)".into(),
        ));
    }
    Ok(name.to_string())
}

pub fn list(pool: &DbPool) -> Result<Vec<TaxDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::list(&conn)?.into_iter().map(TaxDto::from).collect())
}

pub fn create(pool: &DbPool, input: TaxInput) -> Result<TaxDto, CatalogError> {
    let name = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    let id = repo::insert(&conn, &name, input.rate)?;
    repo::find_by_id(&conn, id)?
        .map(TaxDto::from)
        .ok_or(CatalogError::NotFound("tax"))
}

pub fn update(pool: &DbPool, id: i64, input: TaxInput) -> Result<TaxDto, CatalogError> {
    let name = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    repo::update(&conn, id, &name, input.rate)?;
    repo::find_by_id(&conn, id)?
        .map(TaxDto::from)
        .ok_or(CatalogError::NotFound("tax"))
}

pub fn set_active(pool: &DbPool, id: i64, is_active: bool) -> Result<(), CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::set_active(&conn, id, is_active)?;
    Ok(())
}
