# Local API Contract

Implemented in Phase 10 (server) and consumed starting Phase 11 (mobile
client). This document defines the contract now so both sides can be
built against a stable shape.

## Transport

- Base URL: `http://<desktop-lan-ip>:8080` — the Desktop binds to the
  machine's LAN interface (not `0.0.0.0` to the public internet) and
  displays the resolved IP/port in its UI.
- All business endpoints are versioned under `/api/v1`.
- `GET /api/health` is unversioned and unauthenticated — used by Mobile's
  discovery/connection-indicator logic.
- WebSocket endpoint: `ws://<desktop-lan-ip>:8080/api/v1/ws` (auth via the
  same device token, sent as a query param or the first frame).

## Response envelope

Every response is JSON with a consistent shape:

```json
{
  "success": true,
  "data": { },
  "message": "OK"
}
```

```json
{
  "success": false,
  "message": "Product not found",
  "code": "PRODUCT_NOT_FOUND",
  "errors": {}
}
```

`code` is a stable machine-readable string (`VALIDATION_ERROR`,
`UNAUTHORIZED`, `FORBIDDEN`, `DEVICE_REVOKED`, `PAIRING_CODE_EXPIRED`,
`PAIRING_CODE_INVALID`, `RATE_LIMITED`, `IDEMPOTENCY_CONFLICT`, ...).
Internal errors are logged server-side with full detail; the client only
ever sees the generic `INTERNAL_ERROR` code and a safe message.

## HTTP status codes

`200` success · `201` created · `400` validation error · `401` missing/invalid
token · `403` valid token, insufficient permission · `404` not found ·
`409` conflict (e.g. duplicate pairing, stale idempotency key reused with a
different payload) · `422` semantically invalid (e.g. selling below cost
where policy forbids it) · `429` rate limited · `500` internal error.

## Auth & pairing

| Method & Path | Purpose |
|---|---|
| `POST /api/v1/pairing/request` | Desktop UI generates a pairing code (admin-initiated) |
| `POST /api/v1/pairing/claim` | Mobile submits `{ code }`; on success returns `{ device_id, device_token }` |
| `POST /api/v1/auth/login` | `{ username, password }` -> user session (Desktop UI and, if a user also authenticates on a paired device, Mobile) |
| `POST /api/v1/auth/logout` | Invalidate the current session/token |
| `GET /api/v1/devices` | List paired devices (requires `devices.view`) |
| `PATCH /api/v1/devices/:id` | Rename/enable/disable a device (`devices.manage`) |
| `DELETE /api/v1/devices/:id` | Revoke a device's token (`devices.manage`) |

Pairing codes: 6 digits, expire after a short TTL, max N claim attempts,
single-use, never logged or stored in plaintext (hashed like a password).
Device tokens are bearer tokens sent as `Authorization: Bearer <token>` on
every request after pairing; middleware rejects revoked/disabled/expired
tokens before the request reaches any handler.

## Resource endpoints (all under `/api/v1`, all require a valid device token)

| Resource | Endpoints |
|---|---|
| Products | `GET /products`, `GET /products/:id`, `GET /products/barcode/:code` |
| Categories/Brands/Units | `GET /categories`, `GET /brands`, `GET /units` |
| Customers | `GET /customers`, `GET /customers/:id`, `POST /customers` |
| Stock | `GET /stock/:productId` |
| Sales | `GET /sales`, `GET /sales/:id`, `POST /sales` |
| Sync | `GET /sync?since=<ISO8601>&cursor=<opaque>` |

`POST /sales` (checkout) requires an `Idempotency-Key` header
(`<device_id>-SALE-<yyyyMMdd>-<seq>`). A retried request with the same key
and the same payload returns the original result (`200`, not a second
`201`); the same key with a different payload is a `409
IDEMPOTENCY_CONFLICT`. See `synchronization.md` for the full flow.

## Sync response shape

```json
{
  "server_time": "2026-09-18T12:00:00Z",
  "changes": [
    { "entity_type": "product", "entity_id": 42, "operation": "update", "server_version": 118, "payload": { } }
  ],
  "deleted": [
    { "entity_type": "customer", "entity_id": 7, "server_version": 119 }
  ],
  "next_cursor": "119"
}
```

## Middleware stack (applied in order)

1. Request ID + structured logging
2. Rate limiting (per device token; stricter on `/pairing/*` and `/auth/*`)
3. JSON body validation against a schema (reject unknown/malformed fields)
4. Device token authentication
5. Role/permission authorization for the specific route
6. Handler (wrapped so any panic/error becomes the standard error envelope,
   never a raw stack trace)

## Real-time events (WebSocket)

`product.created` · `product.updated` · `product.deleted` · `stock.updated` ·
`sale.created` · `sale.updated` · `customer.updated` · `device.connected` ·
`device.disconnected` · `sync.completed`

Event payload shape:

```json
{ "event": "stock.updated", "data": { "product_id": 42, "current_stock": 17 }, "server_time": "..." }
```

WebSocket is notify-only: a client that reconnects after missing events
must still call `GET /sync?since=...` to guarantee consistency — it must
never rely on having seen every event.
