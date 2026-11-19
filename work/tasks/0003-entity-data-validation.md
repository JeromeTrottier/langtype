---
id: 3
title: Validate and normalise entity data and event payloads
milestone: M1
status: proposed
depends_on: [1]
size: M
area: [server/crates/core/src/validate.rs]
attempts: 0
branch: null
---

## Goal
Invariant 5: entities always validate against their kit type. Implement pure validation in
`core::validate`: `validate_entity_data(&EntityTypeDef, Json) -> Result<Json, Vec<FieldError>>`
returning normalised data, and the same for event payloads and KB settings (DOMAIN_MODEL §6).

## Acceptance criteria
- [ ] Missing required field → `FieldError { field, kind: Missing }` — unit test.
- [ ] Unknown field → error (kits are strict) — unit test.
- [ ] Each base type rejects wrong JSON types (`int` rejects `1.5` and `"1"`, `date` rejects
      non-`YYYY-MM-DD`, `timestamp` requires RFC 3339 UTC) — table-driven unit test.
- [ ] `enum[a, b]` rejects `c`; `list(int)` validates each element and reports the index — tests.
- [ ] `ref(t)` accepts a UUID string and reports it in a returned `refs` list (existence and type
      are checked by the store, 0012) — test.
- [ ] Defaults are applied for absent optional fields — test.
- [ ] `text`/`markdown` values (including inside lists) are NFC-normalised: `"e\u{301}"` becomes
      `"é"` — test.
- [ ] All errors are returned at once, not just the first — test.
- [ ] Property test (proptest): output of a successful validation validates again unchanged
      (idempotent).
- [ ] `just check` passes.

## Notes
Pure functions, no I/O. Errors via `thiserror`. Out of scope: visibility of fields (0004).

## Log
