//! Tauri IPC commands used by the React UI for native/desktop-only
//! functionality (e.g. printing, backups, file dialogs).
//!
//! Business logic for POS operations (sales, stock, etc.) lives in `db/`
//! and is shared between IPC commands here and the local API in `api/`,
//! rather than being duplicated in either layer.
