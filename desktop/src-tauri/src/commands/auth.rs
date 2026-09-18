use tauri::State;

use crate::auth::{self, AuthState, AuthenticatedUser};
use crate::db::DbPool;
use crate::error::CommandError;

#[tauri::command]
pub fn auth_needs_setup(pool: State<'_, DbPool>) -> Result<bool, CommandError> {
    Ok(auth::needs_setup(&pool)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn auth_bootstrap_admin(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    username: String,
    password: String,
    full_name: String,
) -> Result<AuthenticatedUser, CommandError> {
    let user = auth::bootstrap_admin(&pool, &username, &password, &full_name)?;
    *auth_state.0.lock().unwrap() = Some(user.clone());
    Ok(user)
}

#[tauri::command]
pub fn auth_login(
    pool: State<'_, DbPool>,
    auth_state: State<'_, AuthState>,
    username: String,
    password: String,
) -> Result<AuthenticatedUser, CommandError> {
    let user = auth::login(&pool, &username, &password)?;
    *auth_state.0.lock().unwrap() = Some(user.clone());
    Ok(user)
}

#[tauri::command]
pub fn auth_logout(auth_state: State<'_, AuthState>) -> Result<(), CommandError> {
    *auth_state.0.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn auth_current_user(auth_state: State<'_, AuthState>) -> Option<AuthenticatedUser> {
    auth_state.0.lock().unwrap().clone()
}
