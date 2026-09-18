# TijaraPOS

An offline-first Point of Sale ecosystem: a Desktop POS application (the
server and source of truth) paired with a Flutter Mobile companion app
over the local Wi-Fi network. See [`docs/architecture.md`](docs/architecture.md)
for the full design.

```
                    LOCAL WI-FI
                        |
          +-------------+-------------+
          |                           |
   DESKTOP POS                 FLUTTER MOBILE
   React + TS + Tauri   <----> Flutter + GetX
   SQLite (source of truth)    REST + WebSocket
```

- The Desktop app works fully offline and remains functional with no
  mobile devices connected.
- The Mobile app never touches SQLite directly — every read/write goes
  through the Desktop's local, versioned API.

## Repository structure

```
TijaraPOS/
├── desktop/     React + TypeScript + Tauri 2.x desktop app (see desktop/README.md)
├── mobile/      Flutter + Dart + GetX mobile app
└── docs/
    ├── architecture.md      System design and component responsibilities
    ├── database.md          SQLite schema design
    ├── api.md                Local API contract (REST + WebSocket)
    ├── synchronization.md    Incremental sync, offline queue, conflict rules
    └── deployment.md         Setup, commands, and environment notes
```

## Getting started

```bash
# Desktop
cd desktop && npm install && npm run tauri dev

# Mobile
cd mobile && flutter pub get && flutter run
```

Full prerequisites and commands: [`docs/deployment.md`](docs/deployment.md).

## Development phases

Built incrementally; each phase is validated (build, tests, docs) before
the next begins.

| # | Phase | Status |
|---|---|---|
| 1 | Architecture, repository structure, tooling | ✅ Done |
| 2 | SQLite schema + migrations | ✅ Done |
| 3 | Desktop authentication + RBAC | ✅ Done |
| 4 | Product / category management | ✅ Done |
| 5 | POS checkout | ✅ Done |
| 6 | Inventory | ✅ Done |
| 7 | Customers + suppliers | Next |
| 8 | Cash register | Planned |
| 9 | Reports | Planned |
| 10 | Local API | Planned |
| 11 | Mobile pairing / authentication | Planned |
| 12 | Mobile product / catalog | Planned |
| 13 | Mobile sales | Planned |
| 14 | Offline queue + synchronization | Planned |
| 15 | WebSocket real-time events | Planned |
| 16 | Printing | Planned |
| 17 | Backup / restore | Planned |
| 18 | Security hardening | Planned |
| 19 | Testing | Planned |
| 20 | Production packaging and deployment | Planned |

## Phase 1 summary

**Files created:**
- `desktop/` — Vite + React 19 + TypeScript app scaffolded with Tailwind
  CSS v4, a shadcn/ui-style `Button` primitive + `components.json`,
  TanStack Query, React Router, React Hook Form, Zod, and a Tauri 2.x
  backend (`src-tauri/`) with `db/`, `api/`, `commands/` Rust modules
  stubbed for their respective future phases.
- `mobile/` — Flutter 3.47 app scaffolded via `flutter create` (org
  `com.tijarapos`, Android/iOS/Linux targets) with the `core/` `data/`
  `modules/` clean-architecture layout, GetX, Dio, `flutter_secure_storage`,
  `connectivity_plus`, and `mobile_scanner` wired into `pubspec.yaml`.
- `docs/` — architecture, database, API, synchronization, and deployment
  design documents.

**Verified:**
- Desktop: `npm run typecheck`, `npm run lint`, `npm run build` all pass;
  `cargo check` passes for the Rust backend (Linux target).
- Mobile: `flutter analyze` and `flutter test` pass; `flutter build linux
  --debug` succeeds as a compile smoke test in the absence of Android/iOS
  SDKs in this environment (see `docs/deployment.md` for details).

**Known limitations:** no local API yet (Phase 10) — the two apps do not
talk to each other until Phase 11. Mobile Android/iOS builds are
unverified in this container (no SDKs installed here); Desktop
Windows/macOS bundles are unverified (Linux-only container).

## Phase 2 summary

**Files created:**
- `desktop/database/migrations/0001`–`0011` — forward-only SQL migrations
  implementing every table from `docs/database.md`: identity/RBAC,
  devices, pricing rules, catalog, parties, sales, purchases, inventory,
  cash register/expenses, and system tables (settings/audit/sync), plus a
  seed migration for the permission catalog and the five roles.
- `desktop/src-tauri/src/db/migrations.rs` — embeds the migration files
  (`rust-embed`) and applies pending ones transactionally, tracked in a
  `schema_migrations` table.
- `desktop/src-tauri/src/db/mod.rs` — connection pool (`r2d2` +
  `r2d2_sqlite`) with `PRAGMA foreign_keys`/WAL enabled per connection,
  and `db::init()` wired into `lib.rs`'s Tauri `setup()` hook (DB lives in
  the OS app-data directory, managed as Tauri state for future commands).

**Verified:**
- `cargo test` (from `desktop/src-tauri/`): 4 tests passing — full schema
  applies to a fresh database, re-running migrations is a no-op, foreign
  keys are actually enforced (rejecting a product with a non-existent
  `unit_id`), and the RBAC seed data is present with `admin` holding every
  permission.
- `cargo check`, `cargo clippy --all-targets`, and `cargo build` all clean.

**Known limitations:** no repositories/business logic on top of the
schema yet (stock-movement invariants, sale transactions, etc. are
enforced by application code built in Phases 4–8, not by the schema
alone beyond CHECK constraints and FKs). No default admin user is
seeded — Phase 3 creates the first account via a setup flow rather than a
hardcoded credential.

## Phase 3 summary

**Files created:**
- `desktop/src-tauri/src/db/repositories/{users,rbac}.rs` — plain SQL
  data access (find/insert users, resolve a user's effective permissions
  across their roles), usable with both a pooled connection and a
  transaction.
- `desktop/src-tauri/src/auth/` — `password.rs` (Argon2id hashing,
  isolated to one file), `mod.rs` (`needs_setup`, `bootstrap_admin`,
  `login`, `has_permission`/`require_permission`, input validation, and an
  `AuthState` holding the desktop app's one active session). A failed
  login for an unknown username does the same Argon2 work as a wrong
  password, so the two aren't distinguishable by response time.
- `desktop/src-tauri/src/error.rs` — `CommandError { code, message }`,
  the shape every Tauri command error takes; internal errors (SQL
  failures, etc.) are logged server-side and never forwarded to the UI.
- `desktop/src-tauri/src/commands/auth.rs` — `auth_needs_setup`,
  `auth_bootstrap_admin`, `auth_login`, `auth_logout`, `auth_current_user`
  Tauri commands, registered and given managed `AuthState` in `lib.rs`.
- `desktop/src/features/auth/` — `api.ts` (typed, Zod-validated `invoke()`
  wrappers), `AuthProvider.tsx` (TanStack Query-backed `useAuth()`
  context: `user`, `needsSetup`, `login`, `bootstrapAdmin`, `logout`,
  `hasPermission`), `RequirePermission.tsx`, `SetupPage.tsx` and
  `LoginPage.tsx` (React Hook Form + Zod). `App.tsx` now gates on
  `needsSetup`/`user` instead of always showing the dashboard, and
  `DashboardPage` shows the signed-in user, their resolved permissions,
  and a logout button.
- `desktop/src/lib/permissions.ts` — the `PERMISSIONS` catalog (mirroring
  the seeded permission keys) and `PermissionKey` type, so features
  reference `PERMISSIONS.ProductsCreate` instead of a raw string.
- New shadcn-style primitives: `input.tsx`, `label.tsx`, `card.tsx`.

**Verified:**
- `cargo test`: 11 tests passing, including bootstrap-then-login,
  wrong-password vs. unknown-username both rejected identically, disabled
  accounts rejected, and permission resolution. `cargo clippy
  --all-targets` and `cargo build` clean.
- `npm run typecheck`, `lint`, and `build` all pass.
- Ran the actual compiled binary headlessly (`tauri build --debug
  --no-bundle` + `xvfb-run`): the app started, resolved its OS app-data
  directory, ran migrations, and every table from Phase 2 exists in the
  resulting database file — a real end-to-end run, not just unit tests.

**Known limitations:** no protected commands exist yet to exercise
`require_permission` outside of tests (the first ones land in Phase 4);
full interactive click-through of the login/setup UI wasn't performed —
verified via the real backend run above plus frontend build/typecheck,
not by driving the webview with input events.

## Phase 4 summary

**Files created:**
- `desktop/src-tauri/src/db/repositories/{categories,brands,units,taxes,discounts,products}.rs`
  — plain SQL for the catalog tables, matching the Phase 2 schema exactly
  (categories/products have `is_active` + `deleted_at`; brands/units have
  `deleted_at` only; taxes/discounts have `is_active` only — no hard
  delete for either, since products can still reference them).
  `products.rs` also owns `product_barcodes` (replace-all-on-update) and
  a denormalized join (category/brand/unit names) plus `find_by_barcode`.
- `desktop/src-tauri/src/catalog/` — the business-logic layer: input
  validation, turning a `UNIQUE constraint failed` SQLite error into a
  message like "A product with this SKU already exists"
  (`error::friendly_conflict`), and transaction boundaries for
  product+barcodes writes. One module per entity
  (`categories`/`brands`/`units`/`taxes`/`discounts`/`products`).
- `desktop/src-tauri/src/commands/catalog.rs` — 25 thin Tauri command
  wrappers; every one starts with `auth::require_permission_for` (the
  first real consumer of that Phase 3 helper) gated on
  `products.view/create/update/delete`.
- `desktop/src/features/products/` — `api.ts` (Zod-validated `invoke()`
  wrappers), `queries.ts` (TanStack Query hooks + mutations for every
  entity), `ProductsPage.tsx` (search/category filter/show-inactive,
  table with permission-gated edit/delete), `ProductFormDialog.tsx`
  (React Hook Form + Zod: category/brand/unit/tax/discount selects,
  dynamic barcode list, decimal price fields converted to integer cents
  via `lib/money.ts`), and `CatalogSettingsDialog.tsx` (compact
  add/list/deactivate management for categories, brands, units, taxes,
  and discounts).
- `desktop/src/components/layout/AppLayout.tsx` — the first real nav
  shell (Dashboard/Products links + signed-in user + logout), now that
  there's more than one page; wired in as a layout route.
- New shadcn-style primitives: `dialog.tsx`, `select.tsx`, `table.tsx`,
  `badge.tsx`, `checkbox.tsx` (a plain native checkbox, not a Radix
  primitive — native semantics were enough for a single boolean input).

**Verified:**
- `cargo test`: 27 passing. Beyond the usual create/update/validation
  coverage, two tests specifically target the riskiest part of this
  phase — the camelCase (frontend) ↔ snake_case (Rust) boundary — by
  driving Tauri's real IPC dispatch (`tauri::test::get_ipc_response`)
  with the exact JSON a browser `invoke()` call would send, rather than
  calling Rust functions directly: `full_ipc_contract_smoke_test` walks
  bootstrap → create category → create unit → create product (with
  `categoryId`/`unitId`/`purchasePrice`-style keys) → barcode lookup →
  list, and `a_cashier_is_rejected_by_the_real_ipc_dispatch_...` proves
  an unauthenticated session gets `UNAUTHORIZED` from the real command
  dispatcher, not just from a unit test of the permission-check function.
- `cargo clippy --all-targets` and `cargo build` clean.
- `npm run typecheck`, `lint`, and `build` all pass.
- Rebuilt the actual binary and ran it headlessly again (`xvfb-run`):
  starts cleanly, database still initializes correctly.

**Known limitations:** no image upload — `image_path` exists in the
schema/DTO but there's no file picker yet (would need the Tauri dialog
plugin); deferred rather than half-built. Full interactive click-through
of the Products UI wasn't performed for the same reason as Phase 3 (no
way to drive webview input events here); confidence instead comes from
the IPC contract tests above plus frontend build/typecheck.

## Phase 5 summary

**Files created:**
- `desktop/src-tauri/src/db/repositories/{sales,stock_movements,audit_logs}.rs`
  and `products.rs::adjust_stock`: the SQL behind checkout — sale/sale_item/
  payment inserts, sequential `INV-YYYYMMDD-NNNN` invoice numbers computed
  inside the sale transaction, and the append-only stock_movements/
  audit_logs writers.
- `desktop/src-tauri/src/sales/`: the checkout transaction — snapshots
  each product's price/tax/discount at sale time (never trusts a client-
  supplied price), enforces live stock availability, computes per-line
  discount (from the product's assigned discount, or a cashier-entered
  override) and tax, decrements stock and writes a `stock_movements` row
  per line, writes an audit log, all inside one SQLite transaction so a
  failure anywhere (bad product id, insufficient stock, underpayment)
  rolls back every write. Supports mixed payments (paid amount can exceed
  the total; change is `paid - total`, always derivable, not persisted as
  its own column) and an idempotency key (a retried checkout with the
  same key returns the original sale instead of duplicating it — the
  mechanism docs/synchronization.md describes for Phase 14's offline
  queue, implemented here first). `cancel` reverses the same stock
  movements and audit-logs the status change.
- `desktop/src-tauri/src/commands/sales.rs`: `sales_checkout` (permission
  `sales.create`), `sales_list`/`sales_get` (`sales.view`), `sales_cancel`
  (`sales.cancel`).
- `desktop/src/features/sales/`: `api.ts` (Zod-validated wrappers),
  `cartMath.ts` (client-side preview of the same discount/tax formulas,
  used only so the cart shows live running totals — the receipt always
  renders the server's authoritative response, never this preview),
  `CheckoutPage.tsx` (barcode-scan-or-search input — a scan is just fast
  typing ending in Enter, which triggers an exact barcode lookup; typing
  shows a click-to-add results list — editable quantity/discount per
  line, a payment panel supporting multiple payment rows for mixed
  payment, live change/due display, F2/F9/Esc shortcuts),
  `ReceiptView.tsx` (a printable receipt via `window.print()` and
  `@media print` rules added to `styles/globals.css`), and
  `SalesHistoryPage.tsx` (list, view receipt, cancel).
- Nav gained Checkout and Sales entries, each hidden unless the signed-in
  user holds the corresponding permission.

**A real bug caught by the verification, not shipped:** the first
`cargo build` + headless run logged `[r2d2][ERROR] database is locked`
during startup — r2d2 eagerly opens up to `max_size` connections as soon
as the pool is built, and that warm-up raced the migration transaction
for SQLite's single-writer lock. It was self-recovering (`busy_timeout`
made it retry), but not something to ship and explain away. Fixed by
running migrations on their own connection before the pool is
constructed at all (`db::init` in `desktop/src-tauri/src/db/mod.rs`) —
found *because* this phase's verification step is "build the real binary
and watch it start," not just `cargo test`.

**Verified:**
- `cargo test`: 34 passing, including `checkout_computes_tax_and_
  discount_and_deducts_stock`, `checkout_rejects_insufficient_stock_
  with_no_partial_writes` (asserts zero side effects on failure),
  `checkout_is_idempotent`, `checkout_supports_mixed_payments`, and
  `cancel_reverses_stock_and_sets_status`. Extended the Phase 4 IPC-
  contract technique with `checkout_ipc_contract_smoke_test`, which
  drives `sales_checkout`/`sales_get`/`sales_list`/`sales_cancel` through
  Tauri's real dispatcher with the exact camelCase JSON
  `features/sales/api.ts` sends.
- `cargo clippy --all-targets` and `cargo build` clean; `npm run
  typecheck`/`lint`/`build` clean.
- Rebuilt the binary and ran it headlessly twice — once that caught the
  r2d2 race above, once after the fix to confirm a clean startup log.

**Known limitations, by design:** customer selection is deferred to
Phase 7 (every Phase 5 sale is a walk-in sale, `customer_id: null`) —
building a picker against a table with no CRUD yet would be backwards.
Refunds are out of scope; `cancel` (full void, permission `sales.cancel`)
covers "undo a mistake," not partial-item returns. No thermal-printer
integration — `window.print()` on a template is the real, working
receipt output for now; ESC/POS/58mm/80mm driver work is Phase 16.
Keyboard shortcuts cover F2/F9/Esc, not the full configurable set from
the original brief (Delete, +/-) — a later UI/UX pass, not blocking
checkout itself.

## Phase 6 summary

**Files created:**
- `desktop/src-tauri/src/db/repositories/stock_adjustments.rs` (new) and
  extensions to `stock_movements.rs` (a `list` joined with product/user
  names) and `products.rs` (`list_low_stock`).
- `desktop/src-tauri/src/inventory/`: `adjust_stock` — the one place
  manual stock changes happen outside a sale — validates the reason
  (`adjustment`/`return`/`damage`/`transfer`; `sale`/`purchase` are
  system-generated only, rejected here), rejects a delta that would drive
  `current_stock` negative, and in one transaction writes both the
  human-facing `stock_adjustments` row and the `stock_movements` ledger
  row before updating the product and writing an audit log. `list_movements`
  and `low_stock` back the inventory views.
- `desktop/src-tauri/src/commands/inventory.rs`: `inventory_adjust_stock`
  (permission `stock.adjust`), `inventory_list_movements`/
  `inventory_low_stock` (`stock.view`).
- `desktop/src/features/inventory/`: an Inventory page (low-stock table,
  an adjustment form reusing the same product-search pattern as
  checkout, and a recent-movements history table), plus a low-stock
  widget added to the Dashboard.

**Verified:**
- `cargo test`: 40 passing, including `adjust_stock_rejects_a_delta_
  that_would_go_negative` (zero partial writes on failure, same pattern
  as Phase 5's stock check), `low_stock_returns_only_products_at_or_
  below_their_minimum`, and an `inventory_ipc_contract_smoke_test`
  extending the same real-dispatcher technique from Phases 4–5.
- `cargo clippy --all-targets`, `cargo build`, `npm run
  typecheck/lint/build` all clean; rebuilt the binary and ran it
  headlessly to confirm a clean startup log.

**Known, deliberate scope cut:** purchase receiving (the other half of
"Inventory" in the original brief) needs a supplier, and suppliers have
no CRUD yet — that's Phase 7's job by name ("Customers + suppliers").
Building a purchase-receiving flow against a table with no way to create
a supplier would mean faking half the feature; it's deferred rather than
half-built, and revisited once Phase 7 lands. Everything else from the
brief's inventory list — stock in/out, manual adjustment, returns,
damaged goods, stock history, low-stock alerts — is implemented, and
sales' own stock deduction was already done in Phase 5.

**Next phase:** Phase 7 — Customers + suppliers.
