CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL, -- JSON
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Append-only. No role is ever granted delete access to this table.
CREATE TABLE audit_logs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id     INTEGER REFERENCES users (id) ON DELETE SET NULL,
  device_id   INTEGER REFERENCES devices (id) ON DELETE SET NULL,
  action      TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id   TEXT NOT NULL,
  before      TEXT, -- JSON
  after       TEXT, -- JSON
  ip_address  TEXT,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Append-only log that powers incremental sync (GET /api/v1/sync?since=...).
-- `id` is the global, strictly-increasing `server_version` referenced in
-- docs/api.md and docs/synchronization.md — there is no separate column
-- for it.
CREATE TABLE sync_events (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  entity_type TEXT NOT NULL,
  entity_id   TEXT NOT NULL,
  operation   TEXT NOT NULL CHECK (operation IN ('create', 'update', 'delete')),
  payload     TEXT NOT NULL, -- JSON
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_audit_logs_entity ON audit_logs (entity_type, entity_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs (created_at);
CREATE INDEX idx_sync_events_entity ON sync_events (entity_type, id);
