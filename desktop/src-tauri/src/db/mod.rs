//! SQLite connection pool, schema migrations, and repositories.
//!
//! Populated in Phase 2 (SQLite schema + migrations). The database is the
//! single source of truth for the desktop app and is never accessed
//! directly by the mobile app — only through the local API in `api/`.
