//! Forward-only SQL migration runner. Migration files live in
//! `desktop/database/migrations/` (outside this crate, reviewable as plain
//! SQL) and are embedded into the binary at compile time via `rust-embed`
//! so a bundled release never depends on the working directory.

use rusqlite::{params, Connection};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../database/migrations"]
struct MigrationAssets;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("migration filename '{0}' does not match the expected NNNN_name.sql pattern")]
    InvalidFilename(String),
    #[error("migration file '{0}' is not valid UTF-8")]
    InvalidUtf8(String),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

struct Migration {
    version: i64,
    name: String,
    sql: String,
}

fn load_migrations() -> Result<Vec<Migration>, MigrationError> {
    let mut migrations = Vec::new();

    for file_path in MigrationAssets::iter() {
        let (version_str, rest) = file_path
            .split_once('_')
            .ok_or_else(|| MigrationError::InvalidFilename(file_path.to_string()))?;
        let name = rest
            .strip_suffix(".sql")
            .ok_or_else(|| MigrationError::InvalidFilename(file_path.to_string()))?;
        let version: i64 = version_str
            .parse()
            .map_err(|_| MigrationError::InvalidFilename(file_path.to_string()))?;

        let asset = MigrationAssets::get(&file_path)
            .ok_or_else(|| MigrationError::InvalidFilename(file_path.to_string()))?;
        let sql = std::str::from_utf8(&asset.data)
            .map_err(|_| MigrationError::InvalidUtf8(file_path.to_string()))?
            .to_owned();

        migrations.push(Migration {
            version,
            name: name.to_owned(),
            sql,
        });
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

/// Applies every migration whose version is not yet recorded in
/// `schema_migrations`, each inside its own transaction. Returns the
/// number of migrations that were newly applied. Safe to call on every
/// startup: with nothing pending it is a fast no-op.
pub fn run(conn: &mut Connection) -> Result<usize, MigrationError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        );",
    )?;

    let applied: std::collections::HashSet<i64> = conn
        .prepare("SELECT version FROM schema_migrations")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    let mut applied_count = 0;
    for migration in load_migrations()? {
        if applied.contains(&migration.version) {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(&migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            params![migration.version, migration.name],
        )?;
        tx.commit()?;
        applied_count += 1;
    }

    Ok(applied_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_conn() -> Connection {
        Connection::open_in_memory().expect("open in-memory db")
    }

    #[test]
    fn applies_all_migrations_to_a_fresh_database() {
        let mut conn = memory_conn();
        let applied = run(&mut conn).expect("migrations should apply cleanly");
        assert!(applied > 0);

        for table in [
            "users",
            "roles",
            "permissions",
            "role_permissions",
            "user_roles",
            "devices",
            "device_sessions",
            "categories",
            "brands",
            "units",
            "products",
            "product_barcodes",
            "product_prices",
            "customers",
            "suppliers",
            "sales",
            "sale_items",
            "payments",
            "purchases",
            "purchase_items",
            "stock_movements",
            "stock_adjustments",
            "cash_registers",
            "cash_sessions",
            "cash_movements",
            "expenses",
            "discounts",
            "taxes",
            "settings",
            "audit_logs",
            "sync_events",
        ] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    params![table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "expected table `{table}` to exist");
        }
    }

    #[test]
    fn is_idempotent_on_repeated_runs() {
        let mut conn = memory_conn();
        let first_run = run(&mut conn).expect("first run should apply migrations");
        let second_run = run(&mut conn).expect("second run should be a no-op");

        assert!(first_run > 0);
        assert_eq!(second_run, 0);
    }

    #[test]
    fn seeds_the_rbac_catalog() {
        let mut conn = memory_conn();
        run(&mut conn).unwrap();

        let role_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0))
            .unwrap();
        let permission_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM permissions", [], |r| r.get(0))
            .unwrap();
        let admin_permission_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM role_permissions
                 WHERE role_id = (SELECT id FROM roles WHERE name = 'admin')",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(role_count, 5);
        assert!(permission_count > 0);
        assert_eq!(
            admin_permission_count, permission_count,
            "admin should hold every permission"
        );
    }
}
