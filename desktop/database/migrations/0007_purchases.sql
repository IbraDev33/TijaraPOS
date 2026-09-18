CREATE TABLE purchases (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  supplier_id INTEGER NOT NULL REFERENCES suppliers (id) ON DELETE RESTRICT,
  user_id     INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
  status      TEXT NOT NULL DEFAULT 'received'
                CHECK (status IN ('pending', 'received', 'cancelled')),
  total       INTEGER NOT NULL DEFAULT 0 CHECK (total >= 0),
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE purchase_items (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  purchase_id INTEGER NOT NULL REFERENCES purchases (id) ON DELETE CASCADE,
  product_id  INTEGER NOT NULL REFERENCES products (id) ON DELETE RESTRICT,
  quantity    INTEGER NOT NULL CHECK (quantity > 0),
  unit_cost   INTEGER NOT NULL CHECK (unit_cost >= 0),
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_purchases_supplier_id ON purchases (supplier_id);
CREATE INDEX idx_purchase_items_purchase_id ON purchase_items (purchase_id);
CREATE INDEX idx_purchase_items_product_id ON purchase_items (product_id);

CREATE TRIGGER trg_purchases_updated_at
AFTER UPDATE ON purchases
FOR EACH ROW
BEGIN
  UPDATE purchases SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
