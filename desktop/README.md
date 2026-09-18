# TijaraPOS Desktop

React + TypeScript + Tauri 2.x desktop POS application. This is the
server and source of truth for the TijaraPOS system — see the repository
root [`README.md`](../README.md) and [`docs/architecture.md`](../docs/architecture.md)
for the full picture.

## Commands

```bash
npm install
npm run tauri dev    # Full app (Rust backend + webview)
npm run dev          # Vite dev server only

npm run typecheck
npm run lint
npm run format
npm run build         # dist/
npm run tauri build   # native installer for the current OS
```

## Stack

Vite + React 19 + TypeScript, Tailwind CSS v4, a shadcn/ui-style component
system (`components.json`), TanStack Query, React Hook Form + Zod, and a
Rust/Tauri backend for SQLite access, the local API server, and native
integrations (printing, backups).

See [`../docs/deployment.md`](../docs/deployment.md) for prerequisites.
