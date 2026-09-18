use serde::{Deserialize, Serialize};

use super::error::CatalogError;
use crate::db::repositories::discounts as repo;
use crate::db::{DbError, DbPool};

#[derive(Debug, Serialize)]
pub struct DiscountDto {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub value: i64,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct DiscountInput {
    pub name: String,
    pub kind: String,
    pub value: i64,
}

impl From<repo::DiscountRow> for DiscountDto {
    fn from(row: repo::DiscountRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            kind: row.kind,
            value: row.value,
            is_active: row.is_active,
        }
    }
}

fn validate(input: &DiscountInput) -> Result<String, CatalogError> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(CatalogError::Validation(
            "Name must be between 1 and 100 characters".into(),
        ));
    }
    if input.kind != "percent" && input.kind != "fixed" {
        return Err(CatalogError::Validation(
            "Type must be 'percent' or 'fixed'".into(),
        ));
    }
    if input.value < 0 {
        return Err(CatalogError::Validation(
            "Value must not be negative".into(),
        ));
    }
    if input.kind == "percent" && input.value > 100 {
        return Err(CatalogError::Validation(
            "A percent discount cannot exceed 100".into(),
        ));
    }
    Ok(name.to_string())
}

pub fn list(pool: &DbPool) -> Result<Vec<DiscountDto>, CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(repo::list(&conn)?
        .into_iter()
        .map(DiscountDto::from)
        .collect())
}

pub fn create(pool: &DbPool, input: DiscountInput) -> Result<DiscountDto, CatalogError> {
    let name = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    let id = repo::insert(&conn, &name, &input.kind, input.value)?;
    repo::find_by_id(&conn, id)?
        .map(DiscountDto::from)
        .ok_or(CatalogError::NotFound("discount"))
}

pub fn update(pool: &DbPool, id: i64, input: DiscountInput) -> Result<DiscountDto, CatalogError> {
    let name = validate(&input)?;
    let conn = pool.get().map_err(DbError::from)?;
    repo::update(&conn, id, &name, &input.kind, input.value)?;
    repo::find_by_id(&conn, id)?
        .map(DiscountDto::from)
        .ok_or(CatalogError::NotFound("discount"))
}

pub fn set_active(pool: &DbPool, id: i64, is_active: bool) -> Result<(), CatalogError> {
    let conn = pool.get().map_err(DbError::from)?;
    repo::set_active(&conn, id, is_active)?;
    Ok(())
}
