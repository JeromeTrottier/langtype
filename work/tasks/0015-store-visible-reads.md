---
id: 15
title: Visibility-filtered read paths (entities, relations, events, audit log)
milestone: M1
status: proposed
depends_on: [14, 4, 7]
size: M
area: [server/crates/store]
attempts: 0
branch: null
---

## Goal
Invariant 1: visibility is checked server-side on every read path. Every store read takes a
`Viewer` (0004) and returns only what it may see, with hidden fields removed: `get_entity`,
`query_entities(filter)`, `list_relations(entity)`, `timeline(entity?, since?)`,
`list_changesets(since?)`.

## Acceptance criteria
- [ ] `get_entity` of a hidden entity returns `NotFound` (same as nonexistent, no existence leak)
      — store test.
- [ ] `get_entity` redacts role-restricted fields for a viewer without the role, and returns them
      for the KB owner — tests.
- [ ] `query_entities(filter)` applies type, `where` (memory step from 0007), visibility, sort and
      limit, excludes soft-deleted entities, and never counts hidden rows toward `limit` — tests.
- [ ] A `where` on a hidden field never matches for a viewer who can't see that field (no
      inference by filtering) — test.
- [ ] Relations with a hidden end are omitted — test.
- [ ] Timeline returns events newest first, omitting events with any hidden subject (or explicit
      visibility the viewer fails) — test.
- [ ] Audit log lists changesets with changes whose `before`/`after` are redacted for the viewer;
      changes on hidden targets are omitted — test.
- [ ] A test-only "visibility matrix" runs every read path for owner / role A / role B / outsider
      on the `_test` kit and asserts no secret field value ever appears in serialised output.
- [ ] `just check` passes.

## Notes
SQL pre-filtering of `where` comes in 0016; here load candidates by `kb_id` + type and evaluate in
memory (cap rows, report truncation). Text written by people stays plain data here; MCP labelling
(`user_text`) is M2.

## Log
