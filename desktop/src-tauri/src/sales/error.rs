use crate::db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum SalesError {
    #[error("{0}")]
    Validation(String),
    #[error("product not found")]
    ProductNotFound,
    #[error("insufficient stock for {product_name} (have {available}, need {requested})")]
    InsufficientStock {
        product_name: String,
        available: i64,
        requested: i64,
    },
    #[error("sale not found")]
    SaleNotFound,
    #[error("sale is already {0}")]
    InvalidStatusTransition(String),
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}
