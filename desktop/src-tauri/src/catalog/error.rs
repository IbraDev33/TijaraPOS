use crate::db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

/// Turns a `UNIQUE constraint failed: <table>.<column>` SQLite error into
/// a message a user can act on, instead of a generic 500. `mapping` pairs
/// the constraint text (as SQLite reports it) with the friendly message.
pub fn friendly_conflict(err: rusqlite::Error, mapping: &[(&str, &str)]) -> CatalogError {
    if let rusqlite::Error::SqliteFailure(ref sqlite_err, Some(ref message)) = err {
        if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation {
            for (needle, friendly) in mapping {
                if message.contains(needle) {
                    return CatalogError::Conflict((*friendly).to_string());
                }
            }
        }
    }
    CatalogError::Sqlite(err)
}
