---
id: 6
title: Parse and type-check CEL expressions and filters in kit validation
milestone: M1
status: proposed
depends_on:
- 2
size: M
area:
- server/crates/kits
- server/Cargo.toml
attempts: 0
branch: null
issue: 6
issue_synced_at: 2026-11-19T23:09:57.168219Z
issue_body_hash: f6c48a548cf4cdc8
issue_status: proposed
---

## Goal
ADR 0002 and FILTERS.md §Validation: `kit validate` parses and type-checks every expression and
filter against the kit's types and the variables in scope. Pick the Rust CEL crate (PLAN §23
suggests `cel-interpreter`), add it to the workspace, and add a `kits::cel` checker used by
`check_references` for activity `map`/`select` filters and expressions and `import_from` mappings.

## Acceptance criteria
- [ ] A syntactically invalid expression is reported with its location — test.
- [ ] `entity.data.<x>` where `x` is not a field of the filter's type is reported — test. With
      several types, `x` must exist on at least one of them (guarded access with `has()` is the
      author's job) — test; note the rule in a code comment.
- [ ] Comparing a field with a literal of the wrong type (`entity.data.value < "a"` for an `int`)
      is reported — test.
- [ ] Unknown variables are reported per location (`record` outside `import_from`, `event` in a
      filter) — tests.
- [ ] Unknown functions are reported; platform functions (`days_since`, `count`, `has_role`,
      `normalize`, `tts`) and standard ones (`size`, `contains`, `startsWith`, `matches`) are
      accepted — test.
- [ ] `sort.field` must be `created_at`, `updated_at`, `data.<field>` or `state.<name>` — test.
- [ ] `kits/_test` stays valid; a new invalid fixture case covers each problem above — tests.
- [ ] `just check` passes.

## Notes
`state.*` can't be type-checked yet (engine state schemas come with engines); accept any
`state.<name>` path for now and say so in a code comment. Do not change `docs/FILTERS.md` semantics
(needs an ADR) — if the spec is ambiguous, stop and report `blocked` with the question. Record the
CEL crate choice in the PR description.

## Log
- 2026-11-19 18:09 — issue #6 opened
