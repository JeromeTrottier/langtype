---
id: 22
title: REST API for entities, relations, events, audit log and undo
milestone: M1
status: proposed
depends_on: [20]
size: M
area: [server/crates/api, client/src/api/types.gen.ts]
attempts: 0
branch: null
---

## Goal
Expose the KB core through the service layer (0018): entities (create, get, update with
`version`, delete, query with a filter), relations, events (log, timeline), changesets (list,
undo), plus `GET /kbs/{kb}/schema` (the kit's types, roles, events — what `describe_kit` will
return in M2).

## Acceptance criteria
- [ ] `POST /kbs/{kb}/entities` validates data (422 with field errors); `PUT
      /kbs/{kb}/entities/{id}` with a stale `version` → 409 with the current version — tests.
- [ ] `POST /kbs/{kb}/entities/query` takes the FILTERS.md JSON filter; invalid filter → 400 — test.
- [ ] A viewer-access member gets 403 on writes; hidden entities → 404; hidden fields absent from
      every response — tests (reuse the visibility matrix idea from 0015 at the HTTP level).
- [ ] `POST /kbs/{kb}/events` logs an event (engine effects applied); `GET /kbs/{kb}/timeline`
      returns visible events newest first — tests.
- [ ] `GET /kbs/{kb}/changesets` returns the audit log with summaries; `POST
      /kbs/{kb}/changesets/{id}/revert` undoes, 409 on conflict with the later changesets — tests.
- [ ] Writes to a `paused` KB through agent actors and any write to an `upgrading` KB are refused
      with 423 — test.
- [ ] Auth tests for every new route; routes in `ApiDoc`; `just gen-api` run — tests + CI.
- [ ] `just check` passes.

## Notes
Activity routes are 0023. Keep responses compact and paged (`limit`, `cursor`) for lists.

## Log
