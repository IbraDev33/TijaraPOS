//! Thin Tauri wrappers over `catalog::*`. Every command's first line is
//! the permission check — no catalog business logic lives here.

use tauri::State;

use crate::auth::{self, AuthState};
use crate::catalog::brands::{BrandDto, BrandInput};
use crate::catalog::categories::{CategoryDto, CategoryInput};
use crate::catalog::discounts::{DiscountDto, DiscountInput};
use crate::catalog::products::{ProductDto, ProductInput, ProductListQuery};
use crate::catalog::taxes::{TaxDto, TaxInput};
use crate::catalog::units::{UnitDto, UnitInput};
use crate::catalog::{self};
use crate::db::DbPool;
use crate::error::CommandError;

const VIEW: &str = "products.view";
const CREATE: &str = "products.create";
const UPDATE: &str = "products.update";
const DELETE: &str = "products.delete";

// Categories

#[tauri::command]
pub fn categories_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<CategoryDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::categories::list(&pool)?)
}

#[tauri::command]
pub fn categories_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: CategoryInput,
) -> Result<CategoryDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::categories::create(&pool, input)?)
}

#[tauri::command]
pub fn categories_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: CategoryInput,
) -> Result<CategoryDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::categories::update(&pool, id, input)?)
}

#[tauri::command]
pub fn categories_delete(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, DELETE)?;
    Ok(catalog::categories::delete(&pool, id)?)
}

// Brands

#[tauri::command]
pub fn brands_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<BrandDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::brands::list(&pool)?)
}

#[tauri::command]
pub fn brands_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: BrandInput,
) -> Result<BrandDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::brands::create(&pool, input)?)
}

#[tauri::command]
pub fn brands_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: BrandInput,
) -> Result<BrandDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::brands::update(&pool, id, input)?)
}

#[tauri::command]
pub fn brands_delete(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, DELETE)?;
    Ok(catalog::brands::delete(&pool, id)?)
}

// Units

#[tauri::command]
pub fn units_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<UnitDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::units::list(&pool)?)
}

#[tauri::command]
pub fn units_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: UnitInput,
) -> Result<UnitDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::units::create(&pool, input)?)
}

#[tauri::command]
pub fn units_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: UnitInput,
) -> Result<UnitDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::units::update(&pool, id, input)?)
}

#[tauri::command]
pub fn units_delete(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, DELETE)?;
    Ok(catalog::units::delete(&pool, id)?)
}

// Taxes

#[tauri::command]
pub fn taxes_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<TaxDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::taxes::list(&pool)?)
}

#[tauri::command]
pub fn taxes_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: TaxInput,
) -> Result<TaxDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::taxes::create(&pool, input)?)
}

#[tauri::command]
pub fn taxes_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: TaxInput,
) -> Result<TaxDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::taxes::update(&pool, id, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn taxes_set_active(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    is_active: bool,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::taxes::set_active(&pool, id, is_active)?)
}

// Discounts

#[tauri::command]
pub fn discounts_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<DiscountDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::discounts::list(&pool)?)
}

#[tauri::command]
pub fn discounts_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: DiscountInput,
) -> Result<DiscountDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::discounts::create(&pool, input)?)
}

#[tauri::command]
pub fn discounts_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: DiscountInput,
) -> Result<DiscountDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::discounts::update(&pool, id, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn discounts_set_active(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    is_active: bool,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::discounts::set_active(&pool, id, is_active)?)
}

// Products

#[tauri::command]
pub fn products_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    query: Option<ProductListQuery>,
) -> Result<Vec<ProductDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::products::list(&pool, query.unwrap_or_default())?)
}

#[tauri::command]
pub fn products_get(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
) -> Result<ProductDto, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::products::get(&pool, id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn products_get_by_barcode(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    barcode: String,
) -> Result<Option<ProductDto>, CommandError> {
    auth::require_permission_for(&auth_state, VIEW)?;
    Ok(catalog::products::get_by_barcode(&pool, &barcode)?)
}

#[tauri::command]
pub fn products_create(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: ProductInput,
) -> Result<ProductDto, CommandError> {
    auth::require_permission_for(&auth_state, CREATE)?;
    Ok(catalog::products::create(&pool, input)?)
}

#[tauri::command]
pub fn products_update(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    input: ProductInput,
) -> Result<ProductDto, CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::products::update(&pool, id, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn products_set_active(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
    is_active: bool,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, UPDATE)?;
    Ok(catalog::products::set_active(&pool, id, is_active)?)
}

#[tauri::command]
pub fn products_delete(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: i64,
) -> Result<(), CommandError> {
    auth::require_permission_for(&auth_state, DELETE)?;
    Ok(catalog::products::delete(&pool, id)?)
}
