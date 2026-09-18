-- Append-only ledger. `products.current_stock` must never change outside
-- a transaction that also inserts a row here — enforced by only exposing
-- stock changes through a single repository function (Phase 6), not by
-- the schema itself (SQLite has no cross-table write triggers strong
-- enough to guarantee this without hurting legitimate bulk operations).

CREATE TABLE stock_movements (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id     INTEGER NOT NULL REFERENCES products (id) ON DELETE RESTRICT,
  quantity_delta INTEGER NOT NULL CHECK (quantity_delta != 0),
  reason         TEXT NOT NULL
                   CHECK (reason IN ('sale', 'purchase', 'adjustment', 'return', 'damage', 'transfer')),
  reference_type TEXT,
  reference_id   TEXT,
  user_id        INTEGER REFERENCES users (id) ON DELETE SET NULL,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The human-facing record behind a manual correction; each one produces
-- exactly one stock_movements row (reason = 'adjustment').
CREATE TABLE stock_adjustments (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id     INTEGER NOT NULL REFERENCES products (id) ON DELETE RESTRICT,
  quantity_delta INTEGER NOT NULL CHECK (quantity_delta != 0),
  reason         TEXT NOT NULL,
  note           TEXT,
  user_id        INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_stock_movements_product_id_created_at ON stock_movements (product_id, created_at);
CREATE INDEX idx_stock_adjustments_product_id ON stock_adjustments (product_id);
