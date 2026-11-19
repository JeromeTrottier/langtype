---
id: 16
title: Compile the safe CEL subset of filters to SQL
milestone: M1
status: proposed
depends_on: [15]
size: M
area: [server/crates/store]
attempts: 0
branch: null
---

## Goal
FILTERS.md §2 Evaluation step 1: compile the safe subset of `where` (comparisons between
`entity.data.x` / `entity.status` / `entity.updated_at` and a literal; `&&`, `||`, `!`; `in` with a
literal list; `has(entity.data.x)`) to a parameterised SQL predicate, so `query_entities` narrows
rows in the database before the memory step.

## Acceptance criteria
- [ ] Each subset construct compiles to SQL with bound parameters (no string interpolation of
      values) — unit tests on the generated SQL + params.
- [ ] A mixed predicate (`entity.data.hp > 0 && state.in_combat`) compiles the safe conjunct and
      leaves the rest to the memory step; `||` with an unsafe side is not pushed down — tests.
- [ ] Differential property test: for random data and random subset predicates, SQL+memory gives
      the same result set as memory-only evaluation (0007).
- [ ] Injection test: field names and string literals containing quotes, `--`, `;` are safe.
- [ ] Visibility rule from 0015 still holds: a predicate on a field hidden from the viewer is not
      pushed down for that viewer — test.
- [ ] `just check` passes.

## Notes
Use only portable SQL plus `json_extract` behind one helper so the Postgres version can swap it
(store CLAUDE.md). Field names come from the kit, but still validate them against the type's fields.

## Log
