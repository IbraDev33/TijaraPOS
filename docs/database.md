# Database Design

SQLite is the single source of truth, owned exclusively by the Desktop
app. This document is the schema **design** that Phase 2's migrations
implement — it exists so API contracts (`api.md`) and sync rules
(`synchronization.md`) can be written against a stable shape before the
migrations themselves are authored.

## Conventions

- Every table has `id INTEGER PRIMARY KEY` (or a UUID `TEXT` primary key
  for entities created client-side offline, e.g. `sales.id`, so a mobile
  device can generate an ID before it ever reaches the server).
- Every business table has `created_at`, `updated_at` (both UTC
  `TEXT` ISO-8601), and a nullable `deleted_at` for soft deletes on
  syncable entities (products, customers, categories, ...).
- Foreign keys are always declared and enforced (`PRAGMA foreign_keys =
  ON`), with `ON DELETE RESTRICT` by default and `ON DELETE CASCADE` only
  for true ownership relations (e.g. `sale_items` -> `sales`).
- Money is stored as integer minor units (cents) to avoid float rounding;
  formatting to the display currency happens in the UI layer.
- Every write to `products.current_stock` happens inside the same
  transaction as an insert into `stock_movements` — there is no code path
  that changes stock without a movement row. This is enforced by only
  exposing stock changes through a single repository function, never via
  ad hoc `UPDATE products SET current_stock = ...`.

## Tables

### Identity & access
- **users** — id, username, password_hash, full_name, is_active, created_at, updated_at
- **roles** — id, name (admin/manager/cashier/mobile_seller/inventory), description
- **permissions** — id, key (e.g. `products.create`), description
- **role_permissions** — (role_id, permission_id)
- **user_roles** — (user_id, role_id) — a user may hold more than one role

### Devices (mobile pairing)
- **devices** — id, device_name, device_token_hash, status (pending/active/revoked/disabled),
  permissions (JSON snapshot or role_id), created_at, last_seen_at
- **device_sessions** — id, device_id, issued_at, expires_at, revoked_at — supports token
  rotation without losing the device's identity/history

### Catalog
- **categories** — id, name, parent_id, is_active
- **brands** — id, name
- **units** — id, name, abbreviation (e.g. "kg", "pc")
- **products** — id, sku, name, description, category_id, brand_id, unit_id,
  purchase_price, selling_price, tax_id, discount_id, min_stock, current_stock,
  is_active, image_path, created_at, updated_at, deleted_at
- **product_barcodes** — id, product_id, barcode (unique) — supports multiple
  barcodes per product
- **product_prices** — id, product_id, price_list, price, starts_at, ends_at — supports
  time-boxed/tiered pricing without mutating the base `selling_price`

### Parties
- **customers** — id, name, phone, email, address, credit_balance, created_at, updated_at, deleted_at
- **suppliers** — id, name, phone, email, address, created_at, updated_at

### Sales
- **sales** — id (UUID), invoice_number, customer_id, user_id, device_id (nullable —
  null for a Desktop-originated sale), subtotal, discount_total, tax_total, total,
  status (completed/cancelled/refunded), idempotency_key (unique, nullable), created_at
- **sale_items** — id, sale_id, product_id, quantity, unit_price, discount, tax, line_total
- **payments** — id, sale_id, method (cash/card/bank_transfer/other), amount, received_at

### Purchasing
- **purchases** — id, supplier_id, user_id, status, total, created_at
- **purchase_items** — id, purchase_id, product_id, quantity, unit_cost

### Inventory
- **stock_movements** — id, product_id, quantity_delta, reason (sale/purchase/adjustment/
  return/damage/transfer), reference_type, reference_id, user_id, created_at — append-only,
  the audit trail that `products.current_stock` is derived from
- **stock_adjustments** — id, product_id, quantity_delta, reason, note, user_id, created_at
  — the human-facing record; each one produces exactly one `stock_movements` row

### Cash & expenses
- **cash_registers** — id, name, is_active
- **cash_sessions** — id, cash_register_id, user_id, opening_balance, closing_balance,
  expected_cash, actual_cash, difference, opened_at, closed_at
- **cash_movements** — id, cash_session_id, type (sale/withdrawal/deposit/expense),
  amount, note, created_at
- **expenses** — id, cash_session_id, category, amount, note, user_id, created_at

### Pricing rules
- **discounts** — id, name, type (percent/fixed), value, starts_at, ends_at, is_active
- **taxes** — id, name, rate, is_active

### System
- **settings** — key (TEXT PRIMARY KEY), value (JSON), updated_at — business profile
  (name/address/phone for receipts), printer config, feature flags
- **audit_logs** — id, user_id, device_id, action, entity_type, entity_id, before (JSON),
  after (JSON), ip_address, created_at — append-only, no delete permission exposed to any role
- **sync_events** — id, entity_type, entity_id, operation (create/update/delete),
  server_version, payload (JSON), created_at — the append-only log that powers incremental
  sync (`GET /api/v1/sync?since=...`)

## Indexing & constraints (minimum set)

- Unique: `users.username`, `product_barcodes.barcode`, `sales.invoice_number`,
  `sales.idempotency_key`, `devices.device_token_hash`, `role_permissions(role_id, permission_id)`,
  `user_roles(user_id, role_id)`
- Index: `stock_movements(product_id, created_at)`, `sale_items(sale_id)`,
  `sync_events(entity_type, server_version)`, `audit_logs(created_at)`,
  `products(deleted_at)` (partial, for "active products" queries)
- CHECK constraints: `products.current_stock >= 0` is **not** enforced at the DB
  level (backorders/negative-stock policies are a business decision made in
  Phase 6), but `sale_items.quantity > 0`, `payments.amount > 0`, and
  `stock_movements.quantity_delta != 0` are.

## Never

- Never update `products.current_stock` outside a `stock_movements` insert
  in the same transaction.
- Never hard-delete a row that other tables reference by foreign key;
  soft-delete (`deleted_at`) and let sync propagate the tombstone.
- Never let a normal role delete from `audit_logs`.
