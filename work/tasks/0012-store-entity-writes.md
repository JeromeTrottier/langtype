---
id: 12
title: Entity writes through changesets with optimistic locking
milestone: M1
status: proposed
depends_on: [11, 5]
size: M
area: [server/crates/store]
attempts: 0
branch: null
---

## Goal
Invariants 2 and 5: every write is one changeset with before/after, entities always validate, and
updates use optimistic locking on `version`. Add the store's single write path
`Store::apply_changeset(kb, actor, channel, session?, summary, ops)` with entity ops
`CreateEntity`, `UpdateEntity { id, expected_version, data }`, `DeleteEntity { id,
expected_version }` (soft delete).

## Acceptance criteria
- [ ] Create validates `data` against the pinned kit's type (unknown type → error) and writes the
      entity, a `changesets` row (`status = applied`) and a `changes` row with `before = null` in
      one transaction — store test.
- [ ] Update with the current version bumps `version`, sets `updated_at` and records before/after
      — test.
- [ ] Update or delete with a stale version returns `Conflict { current_version }` and writes
      nothing — test.
- [ ] `ref(t)` values must point to a non-deleted entity of type `t` in the **same KB** — tests
      (other KB → error, wrong type → error).
- [ ] Several ops in one call are atomic: if the third fails, the first two are rolled back — test.
- [ ] Writes are refused when the KB is `upgrading`, and for agent actors when `paused` — tests.
- [ ] Delete sets `deleted_at`; the row is kept — test.
- [ ] `just check` passes.

## Notes
Permission checks (`can_edit`) are done by the service layer (0018), not here; the store trusts
its caller but enforces data invariants. Keep the op enum open for relations/events (0013) and
engine state (0018).

## Log
