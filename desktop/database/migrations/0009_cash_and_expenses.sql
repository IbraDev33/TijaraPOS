CREATE TABLE cash_registers (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL UNIQUE,
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE cash_sessions (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  cash_register_id INTEGER NOT NULL REFERENCES cash_registers (id) ON DELETE RESTRICT,
  user_id          INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
  opening_balance  INTEGER NOT NULL CHECK (opening_balance >= 0),
  closing_balance  INTEGER,
  expected_cash    INTEGER,
  actual_cash      INTEGER,
  difference       INTEGER,
  opened_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  closed_at        TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE cash_movements (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  cash_session_id  INTEGER NOT NULL REFERENCES cash_sessions (id) ON DELETE CASCADE,
  type             TEXT NOT NULL CHECK (type IN ('sale', 'withdrawal', 'deposit', 'expense')),
  amount           INTEGER NOT NULL CHECK (amount > 0),
  note             TEXT,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE expenses (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  cash_session_id INTEGER REFERENCES cash_sessions (id) ON DELETE SET NULL,
  category        TEXT NOT NULL,
  amount          INTEGER NOT NULL CHECK (amount > 0),
  note            TEXT,
  user_id         INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_cash_sessions_register_id ON cash_sessions (cash_register_id);
CREATE INDEX idx_cash_sessions_user_id ON cash_sessions (user_id);
CREATE INDEX idx_cash_movements_session_id ON cash_movements (cash_session_id);
CREATE INDEX idx_expenses_session_id ON expenses (cash_session_id);

CREATE TRIGGER trg_cash_registers_updated_at
AFTER UPDATE ON cash_registers
FOR EACH ROW
BEGIN
  UPDATE cash_registers SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
