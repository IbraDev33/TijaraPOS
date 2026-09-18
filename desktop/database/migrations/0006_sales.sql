-- `sales.id` is a client-generatable UUID (TEXT), not an autoincrement
-- integer, so a mobile device can assign a sale's ID before it ever
-- reaches the server (required for the offline queue + idempotency in
-- Phase 14).

CREATE TABLE sales (
  id              TEXT PRIMARY KEY,
  invoice_number  TEXT NOT NULL UNIQUE,
  customer_id     INTEGER REFERENCES customers (id) ON DELETE SET NULL,
  user_id         INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
  device_id       INTEGER REFERENCES devices (id) ON DELETE SET NULL,
  subtotal        INTEGER NOT NULL CHECK (subtotal >= 0),
  discount_total  INTEGER NOT NULL DEFAULT 0 CHECK (discount_total >= 0),
  tax_total       INTEGER NOT NULL DEFAULT 0 CHECK (tax_total >= 0),
  total           INTEGER NOT NULL CHECK (total >= 0),
  status          TEXT NOT NULL DEFAULT 'completed'
                    CHECK (status IN ('completed', 'cancelled', 'refunded')),
  idempotency_key TEXT UNIQUE,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE sale_items (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  sale_id     TEXT NOT NULL REFERENCES sales (id) ON DELETE CASCADE,
  product_id  INTEGER NOT NULL REFERENCES products (id) ON DELETE RESTRICT,
  quantity    INTEGER NOT NULL CHECK (quantity > 0),
  unit_price  INTEGER NOT NULL CHECK (unit_price >= 0),
  discount    INTEGER NOT NULL DEFAULT 0 CHECK (discount >= 0),
  tax         INTEGER NOT NULL DEFAULT 0 CHECK (tax >= 0),
  line_total  INTEGER NOT NULL CHECK (line_total >= 0),
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE payments (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  sale_id     TEXT NOT NULL REFERENCES sales (id) ON DELETE CASCADE,
  method      TEXT NOT NULL CHECK (method IN ('cash', 'card', 'bank_transfer', 'other')),
  amount      INTEGER NOT NULL CHECK (amount > 0),
  received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_sales_customer_id ON sales (customer_id);
CREATE INDEX idx_sales_user_id ON sales (user_id);
CREATE INDEX idx_sales_created_at ON sales (created_at);
CREATE INDEX idx_sale_items_sale_id ON sale_items (sale_id);
CREATE INDEX idx_sale_items_product_id ON sale_items (product_id);
CREATE INDEX idx_payments_sale_id ON payments (sale_id);

CREATE TRIGGER trg_sales_updated_at
AFTER UPDATE ON sales
FOR EACH ROW
BEGIN
  UPDATE sales SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
