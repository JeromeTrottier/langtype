---
id: 2
title: Richer test kit and visibility reference checks
milestone: M1
status: proposed
depends_on: [1]
size: S
area: [kits/_test, server/crates/kits]
attempts: 0
branch: null
---

## Goal
PLAN M1 asks for a minimal test kit that exercises the runtime. Extend `kits/_test` so it covers
everything M1 tests need — role-restricted types, field-level visibility, `reveal_on`,
`editable_by` with `owner`, a `ref` field, a `list` field, an enum, relations and event payloads —
and add the reference checks the validator is still missing for these.

## Acceptance criteria
- [ ] `kits/_test/kit.yaml` has (at least): a type visible only to one role with
      `reveal_on: event(<x>)`; a type with a field visible only to one role; a type with
      `editable_by: [owner, <role>]`; a `ref(..)` field, a `list(..)` field and an `enum[..]` field;
      an event with a payload; a relation. It still validates — existing test.
- [ ] Validator reports `reveal_on` naming an unknown event — test with a new invalid fixture case.
- [ ] Validator reports `editable_by` naming an unknown role (`owner` is allowed) — test.
- [ ] Existing checks without fixture coverage get one: unknown `title_field`/`dedupe_on` field,
      unknown relation endpoint type, `ref(..)` to an unknown type in an event payload — tests.
- [ ] `kits/_test/invalid` (or `kits/_test/invalid-*` folders) cover each new problem and the test
      asserts each message.
- [ ] `kit validate kits/_test` exits 0; on the invalid fixtures exits non-zero — CLI test.
- [ ] `just check` passes.

## Notes
No schema change is needed (all keys already exist in `kit.schema.json`); if one turns out to be,
stop and report `blocked` (schema changes need an ADR). Keep the kit small and generic (no
RPG/language words) — it's a fixture, not a product kit.

## Log
