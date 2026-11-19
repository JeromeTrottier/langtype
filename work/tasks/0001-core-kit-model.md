---
id: 1
title: Typed kit model and domain records in core
milestone: M1
status: proposed
depends_on: []
size: M
area: [server/crates/core, server/crates/kits]
attempts: 0
branch: null
---

## Goal
Give the runtime typed definitions instead of raw `serde_json::Value`: a `KitDef` (roles, entity
types with fields, visibility, `editable_by`, lifecycle, `dedupe_on`; relation types; event types;
engine bindings; activities; briefing; setup; settings) and the core records (`KnowledgeBase`,
`KbMember`, `Entity`, `Relation`, `Event`, `EngineStateRow`) from DOMAIN_MODEL §4 and §6. The
`kits` crate gains `load_kit(dir) -> Result<KitDef, LoadError>` that validates (schema + references)
and then converts. This is the foundation every other M1 task builds on.

## Acceptance criteria
- [ ] `FieldType::parse` handles `text`, `markdown`, `int`, `float`, `bool`, `date`, `timestamp`,
      `json`, `enum[a, b]`, `ref(t)`, `list(T)` (nested) and rejects unknown types — unit tests
      (move the parsing out of `kits::check_field_type`, which then uses it).
- [ ] Short-form (`body: markdown`) and long-form (`{ type, required, default, visibility }`)
      fields convert to the same `FieldDef` — unit test.
- [ ] `VisibilityPolicy` models `all` (default when absent), `roles: [..]` and `reveal_on: event(x)`;
      a malformed `reveal_on` is a load error — unit test.
- [ ] `kits::load_kit(kits/_test)` returns a `KitDef` whose entity types, roles, events, relations,
      engines and activities match the YAML — test in `kits`.
- [ ] `kits::load_kit(kits/_test/invalid)` returns an error carrying every `Problem` — test.
- [ ] Core records (`Entity`, `Relation`, `Event`, `KnowledgeBase`, `KbMember` with `Access` and
      `kit_roles`) round-trip through serde JSON — unit tests.
- [ ] Empty modules `validate`, `visibility`, `changeset`, `filter` are declared in `core/src/lib.rs`
      (doc comment only) so later tasks don't edit `lib.rs` concurrently; `unicode-normalization`
      and `proptest` (dev) are added to `core/Cargo.toml` for the same reason.
- [ ] `just check` passes.

## Notes
Add `RelationId`, `EventId` (exists), `ActivityRunId`, `ConnectionId` id types as needed.
`EntityStatus` (`open | resolved | superseded`) and `Source` enums match the migration CHECKs.
No I/O in core. Out of scope: validation of entity data (0003), visibility logic (0004).

## Log
