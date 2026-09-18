# Synchronization

Desktop is always the source of truth. This document defines the pull
protocol (Desktop -> Mobile), the offline write queue (Mobile -> Desktop),
and the conflict rules between them. Implemented in Phase 14.

## Principles

- **Incremental, not full-dump.** Mobile never re-downloads the whole
  catalog; it asks for everything that changed `since` its last
  successful sync using `GET /api/v1/sync?since=<timestamp>&cursor=<cursor>`.
- **Every syncable table changes through `sync_events`.** Any insert,
  update, or soft-delete to products, prices, categories, customers,
  stock, or sales appends one row to `sync_events` (entity_type,
  entity_id, operation, server_version, payload) in the same transaction
  as the business change. The sync endpoint is a read of this log, not a
  bespoke query per table — this is what makes incremental sync tractable
  and consistent.
- **Monotonic server_version.** `server_version` is a single, global,
  strictly increasing counter (not per-table). `next_cursor` is the last
  `server_version` the client has seen; the next request resumes from
  there. This makes sync resumable and safe to interrupt at any point.
- **Deletes are tombstones.** A soft-deleted row still appears once in
  `changes`/`deleted` with its final `server_version` so clients can
  remove it locally; it is never simply omitted, which would leave stale
  data on the client forever.

## What syncs

Products, prices, categories, customers, stock levels, sales, and sale
status — per the architecture brief. Cash register, expenses, and audit
logs are Desktop-only and are not synced to Mobile.

## Offline write queue (Mobile)

Every mutation Mobile cannot immediately send (network down, or the
Desktop's health check fails) is appended to a local queue instead of
being dropped:

```
operation_id     UUID, generated client-side
device_id
entity_type      e.g. "sale"
entity_id        client-generated UUID for new entities
operation_type   create | update
payload          JSON body that would have been sent to the API
created_at
status           pending | sending | failed | synced
retry_count
```

A background worker drains the queue whenever `GET /api/health` succeeds:
send the oldest `pending` operation, mark it `synced` on `2xx`, bump
`retry_count` and backoff on failure, and surface it to the user (not
silently drop it) once `retry_count` crosses a threshold.

## Idempotency

Every financial operation created offline carries a client-generated
idempotency key: `<device_id>-SALE-<yyyyMMdd>-<seq>`. The Desktop enforces
a unique constraint on `sales.idempotency_key`:

- First arrival: sale is created normally, `201`.
- Same key, identical payload, arrives again (queue retried after a
  timeout that actually succeeded server-side): Desktop returns the
  **original** sale, `200`, and creates nothing new.
- Same key, different payload: `409 IDEMPOTENCY_CONFLICT` — this should
  never happen from a correct client and indicates a bug or tampering, not
  something to silently resolve.

This is what makes "network drops right after the Desktop commits the
sale, Mobile times out and retries" safe: the retry is a no-op, not a
duplicate sale and a double stock deduction.

## Conflict rules

Conflicts are resolved by explicit rule per entity, never by "last write
wins" silently overwriting business data:

| Entity | Rule |
|---|---|
| Products, prices, categories | Desktop always wins. Mobile can display a pending local edit (if ever allowed) but the server's version is authoritative on the next sync. |
| Stock | Desktop is authoritative. Stock changes only ever happen via `stock_movements` on the Desktop side, including movements caused by a mobile-originated sale. |
| Sales created offline | Never discarded. The sale is preserved as submitted; if it cannot be fulfilled as-is (e.g. stock went negative because two devices sold the last unit), the Desktop still creates the sale and stock movement, and the resulting negative/short stock is surfaced to the admin as a reconciliation item rather than silently rejecting a completed customer transaction. |
| Duplicate operations | Idempotency keys, as above — never resolved by guessing which duplicate to keep. |

## Sequence: offline sale, then reconnect

```
Mobile: create sale (Idempotency-Key: DEV1-SALE-20260918-000001)
   -> network unavailable -> queued locally, status=pending
...Wi-Fi restored...
Mobile: GET /api/health -> 200
Mobile: queue worker sends the queued POST /api/v1/sales
Desktop: BEGIN TRANSACTION
         create sale, sale_items, payments
         decrease stock + insert stock_movements
         insert audit_log
         insert sync_events (sale created, stock updated)
         COMMIT
Desktop: broadcasts sale.created, stock.updated over WebSocket
Mobile: marks the queued operation synced; next GET /sync?since=... (or the
        WebSocket event) reconciles local view with the authoritative sale
```

If the transaction fails at any step, the whole thing rolls back and the
API returns an error; the mobile operation stays `pending`/`failed` in the
queue and is retried or surfaced to the user — it is never silently
dropped.
