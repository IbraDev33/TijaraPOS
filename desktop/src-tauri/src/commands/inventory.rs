use tauri::State;

use crate::auth::{self, AuthState};
use crate::db::DbPool;
use crate::error::CommandError;
use crate::inventory::{
    self, LowStockProductDto, MovementDto, MovementListQuery, StockAdjustmentInput,
    StockAdjustmentResult,
};

#[tauri::command]
pub fn inventory_adjust_stock(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    input: StockAdjustmentInput,
) -> Result<StockAdjustmentResult, CommandError> {
    let user = auth::require_permission_for(&auth_state, "stock.adjust")?;
    Ok(inventory::adjust_stock(&pool, &user, input)?)
}

#[tauri::command]
pub fn inventory_list_movements(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    query: Option<MovementListQuery>,
) -> Result<Vec<MovementDto>, CommandError> {
    auth::require_permission_for(&auth_state, "stock.view")?;
    Ok(inventory::list_movements(&pool, query.unwrap_or_default())?)
}

#[tauri::command]
pub fn inventory_low_stock(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
) -> Result<Vec<LowStockProductDto>, CommandError> {
    auth::require_permission_for(&auth_state, "stock.view")?;
    Ok(inventory::low_stock(&pool)?)
}
