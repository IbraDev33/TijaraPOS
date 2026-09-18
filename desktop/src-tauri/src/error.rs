//! The one shape every Tauri command error takes on its way to the
//! frontend: a stable `code` plus a safe `message`. Internal details
//! (SQL errors, panics-turned-errors, etc.) are logged here and never
//! forwarded to the UI.

use serde::Serialize;

use crate::auth::AuthError;
use crate::catalog::CatalogError;
use crate::inventory::InventoryError;
use crate::sales::SalesError;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<AuthError> for CommandError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => {
                CommandError::new("INVALID_CREDENTIALS", "Invalid username or password")
            }
            AuthError::AccountDisabled => {
                CommandError::new("ACCOUNT_DISABLED", "This account has been disabled")
            }
            AuthError::NotAuthenticated => {
                CommandError::new("UNAUTHORIZED", "You must be logged in to do this")
            }
            AuthError::Forbidden => {
                CommandError::new("FORBIDDEN", "You do not have permission to do this")
            }
            AuthError::AlreadySetUp => {
                CommandError::new("ALREADY_SET_UP", "Setup has already been completed")
            }
            AuthError::Validation(message) => CommandError::new("VALIDATION_ERROR", &message),
            AuthError::Internal | AuthError::Db(_) | AuthError::Sqlite(_) => {
                log::error!("auth command failed: {err}");
                CommandError::new("INTERNAL_ERROR", "Something went wrong. Please try again.")
            }
        }
    }
}

impl From<CatalogError> for CommandError {
    fn from(err: CatalogError) -> Self {
        match err {
            CatalogError::NotFound(entity) => {
                CommandError::new("NOT_FOUND", &format!("{entity} not found"))
            }
            CatalogError::Validation(message) => CommandError::new("VALIDATION_ERROR", &message),
            CatalogError::Conflict(message) => CommandError::new("CONFLICT", &message),
            CatalogError::Db(_) | CatalogError::Sqlite(_) => {
                log::error!("catalog command failed: {err}");
                CommandError::new("INTERNAL_ERROR", "Something went wrong. Please try again.")
            }
        }
    }
}

impl From<SalesError> for CommandError {
    fn from(err: SalesError) -> Self {
        match err {
            SalesError::Validation(message) => CommandError::new("VALIDATION_ERROR", &message),
            SalesError::ProductNotFound => {
                CommandError::new("PRODUCT_NOT_FOUND", "One of the products in this sale was not found")
            }
            SalesError::InsufficientStock {
                product_name,
                available,
                requested,
            } => CommandError::new(
                "INSUFFICIENT_STOCK",
                &format!("Not enough stock for {product_name}: {available} available, {requested} requested"),
            ),
            SalesError::SaleNotFound => CommandError::new("NOT_FOUND", "Sale not found"),
            SalesError::InvalidStatusTransition(status) => CommandError::new(
                "INVALID_STATUS",
                &format!("This sale is already {status} and cannot be changed"),
            ),
            SalesError::Db(_) | SalesError::Sqlite(_) => {
                log::error!("sales command failed: {err}");
                CommandError::new("INTERNAL_ERROR", "Something went wrong. Please try again.")
            }
        }
    }
}

impl From<InventoryError> for CommandError {
    fn from(err: InventoryError) -> Self {
        match err {
            InventoryError::Validation(message) => CommandError::new("VALIDATION_ERROR", &message),
            InventoryError::ProductNotFound => {
                CommandError::new("PRODUCT_NOT_FOUND", "Product not found")
            }
            InventoryError::WouldGoNegative {
                product_name,
                current_stock,
                requested_delta,
            } => CommandError::new(
                "STOCK_WOULD_GO_NEGATIVE",
                &format!(
                    "{product_name} only has {current_stock} in stock; a change of {requested_delta} would make it negative"
                ),
            ),
            InventoryError::Db(_) | InventoryError::Sqlite(_) => {
                log::error!("inventory command failed: {err}");
                CommandError::new("INTERNAL_ERROR", "Something went wrong. Please try again.")
            }
        }
    }
}
