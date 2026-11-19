---
id: 14
title: Undo a changeset with a reverting changeset
milestone: M1
status: proposed
depends_on:
- 13
size: M
area:
- server/crates/store
attempts: 0
branch: null
issue: 14
issue_synced_at: 2026-11-19T23:10:29.699523Z
issue_body_hash: 48188a86973e15eb
issue_status: proposed
---

## Goal
DOMAIN_MODEL §7 Undo: `Store::revert_changeset(kb, changeset_id, actor)` creates a new changeset
with `reverts = <id>` whose changes restore `before` (using `core::changeset::plan_revert`), and
appends an `undone` event. Events are never removed.

## Acceptance criteria
- [ ] Reverting an entity create soft-deletes it; an update restores the previous data (version
      still increments); a delete restores the entity — store tests.
- [ ] Reverting a relation create/delete removes/restores it — test.
- [ ] If a later changeset modified the same target, `RevertConflict` lists the later changesets
      and nothing is written — test.
- [ ] Original events stay; one `undone` event referencing the reverted changeset is appended —
      test.
- [ ] Reverting the revert re-applies the original (redo) — test.
- [ ] Changes from another KB's id are refused — test.
- [ ] `just check` passes.

## Notes
Engine state changes (0018) must also be revertible; leave a test-backed `engine_state` branch
for when 0018 lands, or note it for 0018 to add.

## Log
- 2026-11-19 18:10 — issue #14 opened
