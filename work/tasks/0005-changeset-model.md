---
id: 5
title: Changeset model, revert planning and conflict rules in core
milestone: M1
status: proposed
depends_on:
- 1
size: M
area:
- server/crates/core/src/changeset.rs
attempts: 0
branch: null
issue: 5
issue_synced_at: 2026-11-19T23:09:52.988374Z
issue_body_hash: d3faa50d33585461
issue_status: proposed
---

## Goal
Invariant 2: every write goes through one changeset with before/after, and undo is a reverting
changeset (DOMAIN_MODEL §7). Implement the pure part in `core::changeset`: `Changeset`, `Change`
(`target`, `target_id`, `op`, `before`, `after`), `ChangesetStatus`, a builder that records
before/after, and `plan_revert(changeset, later_changes) -> Result<Vec<Change>, RevertConflict>`.

## Acceptance criteria
- [ ] Builder: a create has `before = None`, a delete has `after = None`, an update has both;
      a changeset with no changes can't be built — unit tests.
- [ ] `plan_revert` of a create yields a delete, of a delete a create, of an update an update
      back to `before`, in reverse order — unit tests.
- [ ] `plan_revert` returns `RevertConflict { targets, later_changesets }` when a later applied
      changeset touched the same target — unit test.
- [ ] Reverting a changeset that is itself a revert is allowed (redo); reverting a `draft` or
      `discarded` changeset is an error — unit tests.
- [ ] Property test: applying a changeset's changes and then its planned revert to an in-memory map
      of targets restores the original map.
- [ ] Serde shapes match the `changesets`/`changes` columns (target and op as snake_case strings)
      — unit test.
- [ ] `just check` passes.

## Notes
Drafts (status `draft`, apply re-checks versions) are M2; keep the status enum complete but don't
implement draft application here. Out of scope: storage (0012, 0014).

## Log
- 2026-11-19 18:09 — issue #5 opened
