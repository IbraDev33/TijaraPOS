# core/storage/

`flutter_secure_storage` wrapper for the device token and pairing
credentials, plus the local offline-operation queue (Phase 14). Nothing
here talks to the Desktop's SQLite database directly — the mobile app only
ever reaches it through the local API.
