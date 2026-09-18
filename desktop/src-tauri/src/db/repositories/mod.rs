//! Plain SQL data access. Every function takes `&Connection` so it works
//! identically whether called with a pooled connection or an in-progress
//! transaction (`rusqlite::Transaction` derefs to `Connection`) — the
//! caller decides the transaction boundary, repositories never do.

pub mod brands;
pub mod categories;
pub mod discounts;
pub mod products;
pub mod rbac;
pub mod taxes;
pub mod units;
pub mod users;
