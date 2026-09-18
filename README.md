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
| 3 | Desktop authentication + RBAC | Next |
| 4 | Product / category management | Planned |
| 5 | POS checkout | Planned |
| 6 | Inventory | Planned |
| 7 | Customers + suppliers | Planned |
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

**Known limitations:** no auth yet (Phase 3), no local API yet (Phase 10)
— the two apps do not talk to each other until Phase 11. Mobile
Android/iOS builds are unverified in this container (no SDKs installed
here); Desktop Windows/macOS bundles are unverified (Linux-only
container).

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

**Next phase:** Phase 3 — Desktop authentication + RBAC.
