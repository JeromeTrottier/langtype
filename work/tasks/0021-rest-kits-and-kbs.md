---
id: 21
title: REST API for kits and knowledge bases
milestone: M1
status: proposed
depends_on:
- 11
size: M
area:
- server/crates/api
- client/src/api/types.gen.ts
attempts: 0
branch: null
issue: 21
issue_synced_at: 2026-11-19T23:10:59.077649Z
issue_body_hash: a37f8bf37d065307
issue_status: proposed
---

## Goal
Expose M1's kit runtime over REST so the dashboard (M5) and tests can drive it: at startup install
every valid kit folder under `KITS_DIR` (default `./kits`, skipping `_`-prefixed fixtures unless
`MNEMO_INCLUDE_TEST_KITS=1`); `GET /api/v1/kits`, `GET /api/v1/kits/{id}/{version}`,
`POST /api/v1/kbs`, `GET /api/v1/kbs`, `GET /api/v1/kbs/{kb}`.

## Acceptance criteria
- [ ] Startup installs valid kits and logs (doesn't crash on) invalid ones — integration test with
      a temp kits dir.
- [ ] `GET /kits` lists id, name, description, versions — test.
- [ ] `POST /kbs { name, kit_id, kit_version, settings? }` returns 201 with the pinned version and
      hash; invalid settings → 422 with field errors; unknown kit → 404 — tests.
- [ ] `GET /kbs` lists only the caller's KBs; `GET /kbs/{kb}` for a KB they aren't a member of →
      404 — tests.
- [ ] Every new route: 401 without token, 403/400 on bad `Host`/`Origin` — tests (api CLAUDE.md).
- [ ] Routes are in `ApiDoc`; `just gen-api` run and `types.gen.ts` committed — CI drift check.
- [ ] `KITS_DIR` documented in `.env.example` and PLAN §25 config list.
- [ ] `just check` passes.

## Notes
Handlers stay thin (api CLAUDE.md). Error mapping: one `ApiError` → JSON `{ code, message,
details? }`.

## Log
- 2026-11-19 18:10 — issue #21 opened
