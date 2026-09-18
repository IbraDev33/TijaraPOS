//! The one shape every Tauri command error takes on its way to the
//! frontend: a stable `code` plus a safe `message`. Internal details
//! (SQL errors, panics-turned-errors, etc.) are logged here and never
//! forwarded to the UI.

use serde::Serialize;

use crate::auth::AuthError;

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
