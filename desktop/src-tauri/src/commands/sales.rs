use tauri::State;

use crate::auth::{self, AuthState};
use crate::db::DbPool;
use crate::error::CommandError;
use crate::sales::{self, CheckoutInput, CheckoutResult, SaleDto, SaleListQuery, SaleSummaryDto};

#[tauri::command]
pub fn sales_checkout(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: CheckoutInput,
) -> Result<CheckoutResult, CommandError> {
    let user = auth::require_permission_for(&auth_state, "sales.create")?;
    Ok(sales::checkout(&pool, &user, input)?)
}

#[tauri::command]
pub fn sales_list(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    query: Option<SaleListQuery>,
) -> Result<Vec<SaleSummaryDto>, CommandError> {
    auth::require_permission_for(&auth_state, "sales.view")?;
    Ok(sales::list(&pool, query.unwrap_or_default())?)
}

#[tauri::command]
pub fn sales_get(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: String,
) -> Result<SaleDto, CommandError> {
    auth::require_permission_for(&auth_state, "sales.view")?;
    Ok(sales::get(&pool, &id)?)
}

#[tauri::command]
pub fn sales_cancel(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    id: String,
) -> Result<SaleDto, CommandError> {
    let user = auth::require_permission_for(&auth_state, "sales.cancel")?;
    Ok(sales::cancel(&pool, &id, &user)?)
}
