//! Local HTTP + WebSocket API server exposed to the Flutter mobile app over
//! the local Wi-Fi network (see `docs/api.md`).
//!
//! Populated in Phase 10 (Local API): health check, versioned REST routes
//! under `/api/v1`, device pairing/auth middleware, rate limiting, and the
//! WebSocket event stream. Runs alongside the Tauri window, not inside it.
