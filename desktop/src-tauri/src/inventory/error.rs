use crate::db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("{0}")]
    Validation(String),
    #[error("product not found")]
    ProductNotFound,
    #[error(
        "adjusting {product_name} by {requested_delta} would make stock negative (currently {current_stock})"
    )]
    WouldGoNegative {
        product_name: String,
        current_stock: i64,
        requested_delta: i64,
    },
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}
