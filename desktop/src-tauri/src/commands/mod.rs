//! Tauri IPC commands used by the React UI. These are thin wrappers:
//! business logic for POS operations (auth, sales, stock, etc.) lives in
//! `auth/` and `db/` and is shared between IPC commands here and the
//! local API in `api/`, rather than being duplicated in either layer.

pub mod auth;
