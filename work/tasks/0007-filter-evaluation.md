---
id: 7
title: Filter type and in-memory CEL evaluation
milestone: M1
status: proposed
depends_on:
- 1
- 6
size: M
area:
- server/crates/core/src/filter.rs
- server/crates/core/Cargo.toml
attempts: 0
branch: null
issue: 7
issue_synced_at: 2026-11-19T23:10:00.797272Z
issue_body_hash: 2bc40245722980e6
issue_status: proposed
---

## Goal
The one query format (FILTERS.md §2) as a typed `core::filter::Filter` plus the **memory step**:
evaluate a `where` predicate against an entity (`entity`, `state`, `role`, `user`, `now`) with the
platform functions, then sort and limit. The store (0015, 0016) and engines (0017) use it.

## Acceptance criteria
- [ ] `Filter` deserialises `type` as a string or list, defaults `limit` to 50, rejects `limit`
      outside 1..=1000 and unknown keys — unit tests.
- [ ] `Filter::matches(&EvalCtx)` evaluates comparisons, `&&`, `||`, `!`, `in`, `has()` on
      `entity.data` — table-driven tests.
- [ ] `days_since(t)` counts whole days to `now`; `has_role(r)` reads `role`; `normalize(s)`
      returns NFC + case-folded with accents kept (`"Été"` → `"été"`, `"ё" != "е"`) — tests.
- [ ] A runtime error (missing field without `has`, wrong type) makes the row not match and is
      counted, not a panic — test.
- [ ] `sort` by `updated_at`, `data.<f>` and `state.<f>` with `asc`/`desc`, missing values last,
      stable for ties — tests.
- [ ] Evaluation stops after a configurable row cap and reports `truncated: true` — test.
- [ ] `just check` passes.

## Notes
Use the CEL crate chosen in 0006. `now` comes from the caller (no clock in core). `count(filter)`
and `tts` are out of scope here (rules / presenters later); calling them returns an
"unsupported here" error. Visibility is applied by the caller after filtering (FILTERS.md §2.3).

## Log
- 2026-11-19 18:10 — issue #7 opened
