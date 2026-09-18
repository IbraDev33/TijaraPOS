//! SQLite connection pool, schema migrations, and repositories. This is
//! the single source of truth for the desktop app; the mobile app never
//! accesses it directly, only through the local API in `api/` (Phase 10),
//! which itself calls into the repositories built here.

pub mod migrations;
pub mod repositories;

use std::path::Path;

use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Pool(#[from] r2d2::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] migrations::MigrationError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Opens (creating if necessary) the SQLite database at `db_path`,
/// applies any pending migrations, and returns a ready-to-use connection
/// pool. Every pooled connection has foreign key enforcement and WAL mode
/// turned on, since SQLite does not persist `PRAGMA foreign_keys` in the
/// database file — it must be set per connection.
pub fn init(db_path: &Path) -> Result<DbPool, DbError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )
    });
    let pool = r2d2::Pool::builder().max_size(8).build(manager)?;

    let mut conn = pool.get()?;
    migrations::run(&mut conn)?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_missing_parent_dirs_applies_migrations_and_enforces_fks() {
        let dir = tempfile_dir();
        let db_path = dir.join("nested").join("tijarapos.sqlite3");

        let pool = init(&db_path).expect("init should succeed");
        assert!(db_path.exists());

        let conn = pool.get().unwrap();

        let fk_enabled: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);

        // A product referencing a non-existent unit must be rejected once
        // foreign keys are enforced.
        let result = conn.execute(
            "INSERT INTO products (sku, name, unit_id) VALUES ('SKU-1', 'Test product', 999)",
            [],
        );
        assert!(result.is_err(), "FK violation should be rejected");

        std::fs::remove_dir_all(&dir).ok();
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tijarapos-db-test-{}", uuid_like()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // Avoids pulling in a `uuid`/`rand` dependency just for test isolation.
    fn uuid_like() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
