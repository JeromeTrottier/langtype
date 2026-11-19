---
id: 20
title: "Activity runs: start, idempotent result batches, complete, signals"
milestone: M1
status: proposed
depends_on: [18, 19]
size: M
area: [server/crates/api/src/service, server/crates/store]
attempts: 0
branch: null
---

## Goal
The activity contract (PLAN §14, DOMAIN_MODEL §9): `start_activity` calls the engine's
`next_items` and records an `ActivityRun` with the served items; `post_results(run, batch_id,
results)` passes the batch to `on_activity_result` and applies effects as one changeset
(`channel = activity`, linked to the run); `complete_activity` stores the summary and appends one
`activity_completed` story event; `list_signals` aggregates engine `signals`.

## Acceptance criteria
- [ ] Served items only include entities visible to the user — test with a hidden entity.
- [ ] A result for an item not served in that run is rejected and nothing is applied — test.
- [ ] Re-sending a batch with the same `batch_id` returns the same outcome without applying twice
      — test (counter bumped once).
- [ ] Raw results are stored in `activity_runs.results`, not in events; each item yields at most
      one evidence/story event from the engine — test.
- [ ] Completing appends exactly one `activity_completed` event, sets `status = completed` and
      `ended_at`; completing twice is an error — tests.
- [ ] Completing a run linked to an assignment marks the assignment `done` — test.
- [ ] `list_signals` returns signals from every bound engine, sorted by priority — test.
- [ ] Batch changesets can be undone like any other — test.
- [ ] `just check` passes.

## Notes
Assignments are created by `assign_activity` in M8; seed one directly in the test. Abandoning
runs (timeout) is out of scope.

## Log
