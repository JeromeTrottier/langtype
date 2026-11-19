---
id: 13
title: Relations and append-only events in the write path
milestone: M1
status: proposed
depends_on: [12, 4]
size: M
area: [server/crates/store]
attempts: 0
branch: null
---

## Goal
Complete the write path with relations and events (DOMAIN_MODEL §6, invariant 3): relation ops in
changesets, and `EmitEvent` ops that append validated events referencing their changeset. Events
whose type is named by a `reveal_on` add the event's audience to the subjects' `revealed_to` in
the same changeset.

## Acceptance criteria
- [ ] `CreateRelation` checks the relation type exists and its endpoints match `from`/`to` types
      and are in the same KB; `DeleteRelation` removes it; both record changes — store tests.
- [ ] `EmitEvent` validates the type (kit or platform type) and payload (0003), stores `class`
      from the event type, `subjects`, `actor`, `channel`, `session_id`, `changeset_id` — test.
- [ ] Subjects must be entities of the same KB — test.
- [ ] Events can't be updated or deleted through any store API (no such op); a DB trigger or test
      asserts `UPDATE events` is never issued by the store — test.
- [ ] Logging an event named by `reveal_on` adds its audience (the type's roles, or an explicit
      `reveal_to` in the op) to `revealed_to` of matching subjects, recorded as entity `update`
      changes in the same changeset — test.
- [ ] An event-only changeset (`log_event` with no entity changes) is still one changeset — test.
- [ ] `just check` passes.

## Notes
Platform event types for M1: `entity_revealed`, `undone`, `activity_completed`. Event visibility
defaults to its subjects' (computed on read, 0015); store an explicit `visibility` only when given.

## Log
