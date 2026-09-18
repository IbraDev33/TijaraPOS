-- Paired mobile devices. Pairing codes themselves are not persisted here;
-- they are short-lived, in-memory/hashed state managed by Phase 11's
-- pairing flow. This table is the durable record of devices that
-- completed pairing.

CREATE TABLE devices (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  device_name       TEXT NOT NULL,
  device_token_hash TEXT NOT NULL UNIQUE,
  status            TEXT NOT NULL DEFAULT 'active'
                      CHECK (status IN ('pending', 'active', 'revoked', 'disabled')),
  permissions       TEXT NOT NULL DEFAULT '{}', -- JSON snapshot, or a role reference
  created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  last_seen_at      TEXT
);

CREATE TABLE device_sessions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id  INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
  issued_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  expires_at TEXT NOT NULL,
  revoked_at TEXT
);

CREATE INDEX idx_device_sessions_device_id ON device_sessions (device_id);

CREATE TRIGGER trg_devices_updated_at
AFTER UPDATE ON devices
FOR EACH ROW
BEGIN
  UPDATE devices SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
