# Architecture

## Overview

TijaraPOS is two applications that share one source of truth:

```
                         LOCAL WI-FI
                              |
              +---------------+---------------+
              |                               |
      DESKTOP POS (server)             MOBILE (client)
      React + TS + Tauri 2.x           Flutter + Dart + GetX
      SQLite (source of truth)         no local business DB
      Local HTTP + WebSocket API  <---> Dio + WebSocket client
```

- **Desktop is the server.** It owns the SQLite database, runs the local
  HTTP + WebSocket API, and remains fully functional with zero mobile
  devices connected and zero internet access.
- **Mobile is a thin client.** It never touches SQLite. Every read and
  write goes through the versioned local API (`/api/v1/...`); when the
  network drops, writes are queued locally (see `synchronization.md`) and
  replayed once connectivity returns.
- **No cloud dependency.** Nothing in the critical path (checkout, stock,
  cash register) calls out to the internet. Cloud backup/sync, if ever
  added, is strictly additive and out of scope unless explicitly
  requested.
- **No duplicated business logic.** Rules that decide whether a sale is
  valid, whether stock can go negative, how discounts apply, etc. live in
  the Desktop/Rust + SQLite layer. The Rust layer exposes the same
  functions to Tauri IPC commands (used by the React UI) and to the local
  API (used by Mobile), so there is exactly one implementation of each
  rule, not two.

## Desktop application

```
desktop/
├── src/                  React + TypeScript UI
│   ├── api/              Typed client for the local API (consumed by TanStack Query)
│   ├── components/ui/    shadcn/ui-style primitives (cva + Radix + Tailwind)
│   ├── features/         Feature modules: auth, products, pos, inventory,
│   │                     customers, cash-register, reports, devices, settings
│   ├── hooks/             Cross-feature hooks (permissions, shortcuts, ...)
│   ├── lib/               utils, TanStack Query client
│   ├── providers/         App-wide providers (QueryClientProvider, Router)
│   ├── routes/            Route tree
│   ├── styles/            Tailwind entry + design tokens (light/dark)
│   └── types/             Shared TS types / Zod schemas mirroring the API
├── src-tauri/            Rust backend
│   ├── src/db/            SQLite pool, migrations, repositories (Phase 2)
│   ├── src/api/           Local HTTP + WebSocket server (Phase 10)
│   └── src/commands/      Tauri IPC commands for native-only features
│                          (printing, backup/restore, file dialogs)
├── database/             SQL migration files (Phase 2)
└── tests/                Rust + frontend tests
```

Key stack choices and why:

| Concern | Choice | Why |
|---|---|---|
| Shell | Tauri 2.x | Small, native, offline-capable binaries; Rust backend for real DB/filesystem/network access |
| UI | React + TypeScript + Vite | Fast dev loop, strong typing shared with Zod schemas |
| Styling | Tailwind CSS v4 + shadcn/ui pattern | Consistent design tokens, light/dark for free, no heavy component runtime |
| Server state | TanStack Query | Caching/retries/optimistic updates against the local API, same mental model whether the "server" is 5ms away over IPC or the local API |
| Forms | React Hook Form + Zod | Single schema drives both client validation and (mirrored) server validation |
| Data | SQLite (via `rusqlite`/`sqlx`, decided in Phase 2) | Embedded, transactional, zero-ops, sufficient for single-location retail |

## Mobile application

```
mobile/
├── lib/
│   ├── core/
│   │   ├── network/     Dio client, auth interceptor, discovery/connectivity
│   │   ├── storage/     flutter_secure_storage wrapper, offline queue store
│   │   ├── theme/       Light/dark ThemeData
│   │   ├── constants/   App-wide constants (API prefix, ports, ...)
│   │   └── utils/       Formatters, validators
│   ├── data/
│   │   ├── models/       DTOs mirroring API contracts
│   │   ├── services/     One Dio-backed service per API resource
│   │   └── repositories/ Online/offline-queue decision layer consumed by GetX controllers
│   ├── modules/
│   │   ├── auth/ pairing/ products/ sales/ customers/ stock/ profile/
│   └── main.dart
├── test/                 Widget/unit tests
└── integration_test/     End-to-end flows (pairing, offline sale + sync)
```

GetX is used for state management, dependency injection (`Get.put`/`Get.find`
for repositories/controllers) and navigation, per the architecture brief.
Business rules are **not** re-implemented on mobile: the app renders what
the API returns and queues writes it cannot yet send; the Desktop remains
the only place that decides whether a sale is valid, how stock moves, etc.

## Local network communication

- **REST (`/api/v1/...`)** is the source of truth for data retrieval and
  for writes that need a definitive success/failure response (e.g.
  completing a sale).
- **WebSocket** carries real-time notifications only (`stock.updated`,
  `sale.created`, `device.connected`, ...). A client that misses a
  WebSocket event still ends up consistent on its next REST fetch/sync —
  WebSocket is a convenience for immediate UI refresh, never the only way
  data reaches a client.

See `api.md` for the full contract, `database.md` for the schema behind
it, and `synchronization.md` for how Mobile stays consistent with Desktop
across network interruptions.

## Security boundaries

- Mobile devices are unauthenticated strangers until they complete
  **pairing** (short-lived, single-use, rate-limited code) and are issued a
  **device token**. Every API request after that must carry a valid,
  non-revoked device token.
- The Desktop never trusts data coming from Mobile: every request is
  re-validated and re-authorized server-side (role/permission check),
  regardless of what the UI already enforced.
- Passwords are hashed (Argon2id, decided in Phase 3); device tokens are
  random, high-entropy, stored hashed at rest, and revocable individually.
- All SQL is parameterized; there is no string-built SQL anywhere in the
  codebase.

## Development phases

This repository is built incrementally; see the root `README.md` for the
full 20-phase roadmap and current status. Do not assume a phase's feature
exists until its section of the roadmap is marked done.
