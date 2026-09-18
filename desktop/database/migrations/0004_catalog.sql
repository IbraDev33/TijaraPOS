-- Product catalog. Money columns are integers in minor currency units
-- (cents) to avoid floating-point rounding in totals.

CREATE TABLE categories (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL,
  parent_id  INTEGER REFERENCES categories (id) ON DELETE SET NULL,
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  deleted_at TEXT
);

CREATE TABLE brands (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  deleted_at TEXT
);

CREATE TABLE units (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT NOT NULL,
  abbreviation TEXT NOT NULL UNIQUE,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  deleted_at   TEXT
);

CREATE TABLE products (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  sku            TEXT NOT NULL UNIQUE,
  name           TEXT NOT NULL,
  description    TEXT,
  category_id    INTEGER REFERENCES categories (id) ON DELETE SET NULL,
  brand_id       INTEGER REFERENCES brands (id) ON DELETE SET NULL,
  unit_id        INTEGER REFERENCES units (id) ON DELETE RESTRICT,
  purchase_price INTEGER NOT NULL DEFAULT 0 CHECK (purchase_price >= 0),
  selling_price  INTEGER NOT NULL DEFAULT 0 CHECK (selling_price >= 0),
  tax_id         INTEGER REFERENCES taxes (id) ON DELETE SET NULL,
  discount_id    INTEGER REFERENCES discounts (id) ON DELETE SET NULL,
  min_stock      INTEGER NOT NULL DEFAULT 0 CHECK (min_stock >= 0),
  current_stock  INTEGER NOT NULL DEFAULT 0,
  is_active      INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
  image_path     TEXT,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  deleted_at     TEXT
);

CREATE TABLE product_barcodes (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,
  barcode    TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Time-boxed/tiered pricing without mutating products.selling_price.
CREATE TABLE product_prices (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,
  price_list TEXT NOT NULL DEFAULT 'default',
  price      INTEGER NOT NULL CHECK (price >= 0),
  starts_at  TEXT,
  ends_at    TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_categories_parent_id ON categories (parent_id);
CREATE INDEX idx_products_category_id ON products (category_id);
CREATE INDEX idx_products_brand_id ON products (brand_id);
CREATE INDEX idx_products_deleted_at ON products (deleted_at);
CREATE INDEX idx_product_barcodes_product_id ON product_barcodes (product_id);
CREATE INDEX idx_product_prices_product_id ON product_prices (product_id);

CREATE TRIGGER trg_categories_updated_at
AFTER UPDATE ON categories
FOR EACH ROW
BEGIN
  UPDATE categories SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;

CREATE TRIGGER trg_brands_updated_at
AFTER UPDATE ON brands
FOR EACH ROW
BEGIN
  UPDATE brands SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;

CREATE TRIGGER trg_units_updated_at
AFTER UPDATE ON units
FOR EACH ROW
BEGIN
  UPDATE units SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;

CREATE TRIGGER trg_products_updated_at
AFTER UPDATE ON products
FOR EACH ROW
BEGIN
  UPDATE products SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = OLD.id;
END;
