//! Product catalog business logic: categories, brands, units, taxes,
//! discounts, and products. Thin orchestration over `db::repositories` —
//! validation, transaction boundaries, and turning a raw SQLite error
//! (e.g. a duplicate SKU) into a message a user can act on. Tauri
//! commands (`commands/catalog.rs`) and, from Phase 10, the local API
//! both call into these functions rather than re-implementing any of it.

pub mod brands;
pub mod categories;
pub mod discounts;
pub mod products;
pub mod taxes;
pub mod units;

mod error;
pub use error::CatalogError;

#[cfg(test)]
mod tests {
    use crate::auth::{self, AuthState};
    use crate::db::repositories::{rbac, users};
    use crate::db::DbPool;

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-catalog-perm-test-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        let pool = crate::db::init(&dir.join("test.sqlite3")).unwrap();
        (pool, dir)
    }

    fn unique() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// This is exactly the check `commands/catalog.rs` runs before every
    /// mutating command — proved here against the real seeded roles
    /// rather than a hand-built permission set.
    #[test]
    fn a_cashier_cannot_create_a_product_but_can_view_and_sell() {
        let (pool, dir) = test_pool();
        auth::bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();

        let conn = pool.get().unwrap();
        let cashier_id = users::insert(&conn, "cashier1", "unused-hash", "Cashier One").unwrap();
        let cashier_role_id = rbac::role_id_by_name(&conn, "cashier").unwrap().unwrap();
        rbac::assign_role(&conn, cashier_id, cashier_role_id).unwrap();
        let permissions = rbac::permissions_for_user(&conn, cashier_id).unwrap();
        drop(conn);

        let cashier = auth::AuthenticatedUser {
            id: cashier_id,
            username: "cashier1".into(),
            full_name: "Cashier One".into(),
            permissions: permissions.into_iter().collect(),
        };
        let state = AuthState(std::sync::Mutex::new(Some(cashier)));

        assert!(auth::require_permission_for(&state, "products.view").is_ok());
        assert!(auth::require_permission_for(&state, "sales.create").is_ok());
        assert!(matches!(
            auth::require_permission_for(&state, "products.create"),
            Err(auth::AuthError::Forbidden)
        ));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_unauthenticated_state_is_rejected_before_the_permission_check() {
        let state = AuthState::default();

        assert!(matches!(
            auth::require_permission_for(&state, "products.view"),
            Err(auth::AuthError::NotAuthenticated)
        ));
    }
}
