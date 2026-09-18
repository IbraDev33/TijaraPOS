//! Authentication and permission resolution. This is the single
//! implementation of "who is this and what can they do" — Tauri commands
//! (`commands/auth.rs`) and, from Phase 10, the local API both call into
//! these functions rather than re-implementing the checks.

mod password;

use std::collections::HashSet;
use std::sync::Mutex;

use serde::Serialize;

use crate::db::repositories::{rbac, users};
use crate::db::{DbError, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid username or password")]
    InvalidCredentials,
    #[error("this account has been disabled")]
    AccountDisabled,
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("insufficient permissions")]
    Forbidden,
    #[error("setup has already been completed")]
    AlreadySetUp,
    #[error("{0}")]
    Validation(String),
    #[error("internal error")]
    Internal,
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub permissions: Vec<String>,
}

/// The desktop app's single active session. A Tauri window is one user at
/// a time, so this is simpler than a session table — device sessions
/// (Phase 11) and the local API's bearer tokens (Phase 10) are a separate
/// concern layered on top of the same `login`/permission-resolution logic.
#[derive(Default)]
pub struct AuthState(pub Mutex<Option<AuthenticatedUser>>);

pub fn needs_setup(pool: &DbPool) -> Result<bool, AuthError> {
    let conn = pool.get().map_err(DbError::from)?;
    Ok(users::count(&conn)? == 0)
}

/// Creates the first user account, granted the `admin` role. Only
/// succeeds while the `users` table is empty — there is deliberately no
/// hardcoded default admin credential anywhere in this codebase.
pub fn bootstrap_admin(
    pool: &DbPool,
    username: &str,
    password: &str,
    full_name: &str,
) -> Result<AuthenticatedUser, AuthError> {
    let username = validate_username(username)?;
    validate_password(password)?;
    let full_name = validate_full_name(full_name)?;

    let mut conn = pool.get().map_err(DbError::from)?;
    if users::count(&conn)? > 0 {
        return Err(AuthError::AlreadySetUp);
    }

    let password_hash = password::hash_password(password)?;

    let tx = conn.transaction()?;
    let user_id = users::insert(&tx, &username, &password_hash, &full_name)?;
    let admin_role_id = rbac::role_id_by_name(&tx, "admin")?
        .expect("the 'admin' role must exist — seeded by migration 0011");
    rbac::assign_role(&tx, user_id, admin_role_id)?;
    tx.commit()?;

    let permissions = rbac::permissions_for_user(&conn, user_id)?;
    Ok(AuthenticatedUser {
        id: user_id,
        username,
        full_name,
        permissions: sorted(permissions),
    })
}

pub fn login(
    pool: &DbPool,
    username: &str,
    password: &str,
) -> Result<AuthenticatedUser, AuthError> {
    let conn = pool.get().map_err(DbError::from)?;
    let username = username.trim().to_lowercase();
    let found = users::find_by_username(&conn, &username)?;

    let user = match &found {
        Some(user) => {
            if password::verify_password(password, &user.password_hash) {
                found
            } else {
                None
            }
        }
        None => {
            // Do the same amount of cryptographic work as the "user
            // exists but password is wrong" path, so a nonexistent
            // username isn't measurably faster to reject.
            let _ = password::hash_password(password);
            None
        }
    };

    let user = user.ok_or(AuthError::InvalidCredentials)?;
    if !user.is_active {
        return Err(AuthError::AccountDisabled);
    }

    let permissions = rbac::permissions_for_user(&conn, user.id)?;
    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username,
        full_name: user.full_name,
        permissions: sorted(permissions),
    })
}

pub fn has_permission(user: &AuthenticatedUser, key: &str) -> bool {
    user.permissions.iter().any(|p| p == key)
}

pub fn require_permission(user: &AuthenticatedUser, key: &str) -> Result<(), AuthError> {
    if has_permission(user, key) {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

/// The single check every protected Tauri command (and, from Phase 10,
/// every protected API route) performs first: is anyone logged in, and do
/// they hold this permission. Returns the current user so the caller can
/// use it (e.g. to stamp an audit log) without a second lookup.
pub fn require_permission_for(
    state: &AuthState,
    key: &str,
) -> Result<AuthenticatedUser, AuthError> {
    let user = state
        .0
        .lock()
        .unwrap()
        .clone()
        .ok_or(AuthError::NotAuthenticated)?;
    require_permission(&user, key)?;
    Ok(user)
}

fn sorted(set: HashSet<String>) -> Vec<String> {
    let mut values: Vec<String> = set.into_iter().collect();
    values.sort();
    values
}

fn validate_username(username: &str) -> Result<String, AuthError> {
    let trimmed = username.trim().to_lowercase();
    if trimmed.len() < 3 || trimmed.len() > 50 {
        return Err(AuthError::Validation(
            "Username must be between 3 and 50 characters".into(),
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
    {
        return Err(AuthError::Validation(
            "Username may only contain letters, numbers, '.', '_', and '-'".into(),
        ));
    }
    Ok(trimmed)
}

fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::Validation(
            "Password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

fn validate_full_name(full_name: &str) -> Result<String, AuthError> {
    let trimmed = full_name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err(AuthError::Validation(
            "Full name must be between 1 and 100 characters".into(),
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> (DbPool, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("tijarapos-auth-test-{}", unique()));
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

    #[test]
    fn bootstrap_admin_creates_first_user_with_full_permissions() {
        let (pool, dir) = test_pool();

        let user = bootstrap_admin(&pool, "Admin", "supersecret123", "  Admin User  ").unwrap();

        assert_eq!(user.username, "admin");
        assert_eq!(user.full_name, "Admin User");
        assert!(user.permissions.contains(&"settings.manage".to_string()));
        assert!(user.permissions.contains(&"devices.manage".to_string()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bootstrap_admin_fails_once_a_user_already_exists() {
        let (pool, dir) = test_pool();
        bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();

        let result = bootstrap_admin(&pool, "someoneelse", "supersecret123", "Someone Else");

        assert!(matches!(result, Err(AuthError::AlreadySetUp)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bootstrap_admin_rejects_weak_passwords() {
        let (pool, dir) = test_pool();

        let result = bootstrap_admin(&pool, "admin", "short", "Admin User");

        assert!(matches!(result, Err(AuthError::Validation(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn login_succeeds_with_correct_credentials_case_insensitive_username() {
        let (pool, dir) = test_pool();
        bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();

        let user = login(&pool, "ADMIN", "supersecret123").unwrap();

        assert_eq!(user.username, "admin");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn login_fails_with_wrong_password_or_unknown_username_identically() {
        let (pool, dir) = test_pool();
        bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();

        let wrong_password = login(&pool, "admin", "wrongpassword");
        let unknown_user = login(&pool, "nobody", "whatever12345");

        assert!(matches!(wrong_password, Err(AuthError::InvalidCredentials)));
        assert!(matches!(unknown_user, Err(AuthError::InvalidCredentials)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn login_rejects_disabled_accounts() {
        let (pool, dir) = test_pool();
        bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute(
                "UPDATE users SET is_active = 0 WHERE username = 'admin'",
                [],
            )
            .unwrap();
        }

        let result = login(&pool, "admin", "supersecret123");

        assert!(matches!(result, Err(AuthError::AccountDisabled)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn permission_helpers_reflect_the_users_resolved_permissions() {
        let (pool, dir) = test_pool();
        let user = bootstrap_admin(&pool, "admin", "supersecret123", "Admin User").unwrap();

        assert!(has_permission(&user, "products.view"));
        assert!(require_permission(&user, "products.view").is_ok());

        let mut stripped = user.clone();
        stripped.permissions.clear();
        assert!(!has_permission(&stripped, "products.view"));
        assert!(matches!(
            require_permission(&stripped, "products.view"),
            Err(AuthError::Forbidden)
        ));

        std::fs::remove_dir_all(&dir).ok();
    }
}
