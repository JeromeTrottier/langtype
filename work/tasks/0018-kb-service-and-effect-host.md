---
id: 18
title: KB service layer and effect host (validate + apply in one transaction)
milestone: M1
status: proposed
depends_on:
- 17
- 16
- 21
size: M
area:
- server/crates/api/src/service
- server/crates/store
attempts: 0
branch: null
issue: 18
issue_synced_at: 2026-11-19T23:10:46.449542Z
issue_body_hash: 0c942490085fe8fd
issue_status: proposed
---

## Goal
The one service layer (PLAN §12) that every caller (REST now, MCP in M2) uses: resolve the actor's
`Viewer`, check `can_edit` for writes, call the store's write path, run engine hooks
(`on_event` for events in the changeset, `on_tool_call`), validate returned effects against the
kit schema and the **caller's** edit rights, and apply them **in the same transaction**
(DOMAIN_MODEL §13 "Agent writes during a session"). Engine state changes are recorded as
`engine_state` changes so they're audited and undoable.

## Acceptance criteria
- [ ] `KbService::upsert_entity` by a member without edit rights → `Forbidden`, nothing written
      — test.
- [ ] Logging an event subscribed to by a test engine applies the engine's `SetState` effect in the
      same changeset (one changeset row, engine_state change with before/after) — test.
- [ ] An effect that fails validation (unknown type, invalid data, entity the caller can't edit)
      rolls back the whole operation, including the triggering write — test.
- [ ] `SetState` uses optimistic locking on `engine_state.version` — conflict test.
- [ ] Undo (0014) of a changeset with engine_state changes restores the previous state — test.
- [ ] Effect limits: more than N effects per hook or an effect payload over M KB is rejected — test.
- [ ] Engine hooks recurse at most one level (effects' events don't re-trigger `on_event`
      indefinitely) — test with an engine that emits the event it listens to.
- [ ] `just check` passes.

## Notes
Engines get a `Ctx` backed by the store with the caller's visibility. Put the service in `api`
(`src/service/`) unless a dedicated crate is clearly better — note the choice in the PR and in
PLAN §24 if a crate is added. `Schedule` effects are stored nowhere yet (on_tick is later): reject
them with a clear error for now and note it.

## Log
- 2026-11-19 18:10 — issue #18 opened
