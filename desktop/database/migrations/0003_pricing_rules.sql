-- Discounts and taxes are created before the catalog (0004) because
-- products reference them.

CREATE TABLE discounts (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL,
  type       TEXT NOT NULL CHECK (type IN ('percent', 'fixed')),
  value      INTEGER NOT NULL CHECK (value >= 0),
  starts_at  TEXT,
  ends_at    TEXT,
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE taxes (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL,
  rate       REAL NOT NULL CHECK (rate >= 0),
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TRIGGER trg_discounts_updated_at
AFTER UPDATE ON discounts
FOR EACH ROW
BEGIN
  UPDATE discounts SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;

CREATE TRIGGER trg_taxes_updated_at
AFTER UPDATE ON taxes
FOR EACH ROW
BEGIN
  UPDATE taxes SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
