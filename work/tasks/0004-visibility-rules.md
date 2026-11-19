---
id: 4
title: Visibility and edit rules in core
milestone: M1
status: proposed
depends_on: [1]
size: M
area: [server/crates/core/src/visibility.rs]
attempts: 0
branch: null
---

## Goal
Invariant 1: visibility is decided server-side by one function set. Implement DOMAIN_MODEL §10 in
`core::visibility`: a `Viewer { user_id, access, kit_roles }`, `can_see`, `visible_fields`/`redact`,
`can_edit`, `relation_visible`, `event_visible` and `reveals_for(event)` (which audiences a
`reveal_on` event adds).

## Acceptance criteria
- [ ] KB `owner` access sees every entity and field — unit test.
- [ ] A role-restricted type is hidden from a viewer without that role, and visible once
      `revealed_to` contains their role or user id — unit tests.
- [ ] `owner_user_id == viewer.user_id` grants visibility — unit test.
- [ ] `redact` removes fields whose field-level visibility fails, and keeps the rest — unit test.
- [ ] `can_edit` requires `can_see` and `editable_by` (with `owner` meaning `owner_user_id`);
      `viewer` access can never edit — unit tests.
- [ ] A relation is visible only if both ends are — unit test.
- [ ] An event with no explicit visibility is visible only if all its subjects are visible
      ("most restrictive") — unit test.
- [ ] `reveals_for(event_type)` returns the entity types whose `reveal_on` names that event — test.
- [ ] Property tests (proptest): `redact` never returns a field the viewer can't see; `can_edit`
      implies `can_see`; owner always sees everything.
- [ ] `just check` passes.

## Notes
An agent viewer is the member's viewer intersected with connection scopes (read-only scope →
`can_edit` false); model it now as `Viewer::with_scopes`. Test with types from the 0001 model, not
with the `_test` kit files (no I/O in core).

## Log
