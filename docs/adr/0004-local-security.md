# 0004. Local-mode security

- Status: accepted
- Date: 2026-11-18

## Context
A local HTTP server can be reached by any website the user visits (cross-site requests, DNS
rebinding). The MCP spec requires Origin validation for local HTTP servers.

## Decision
- Bind to 127.0.0.1 only.
- Reject requests whose `Host` isn't `localhost:<port>` / `127.0.0.1:<port>`.
- Reject requests with an `Origin` outside the allowed local origins (server itself, Vite dev).
- Protected routes require `Authorization: Bearer <token>`; the token is generated on first run
  in `<DATA_DIR>/local_token` (mode 600) and added automatically by plugins, the stdio bridge and
  the dashboard.

## Consequences
Every new route is protected unless explicitly public (health only, for now). Tests cover each rule.
