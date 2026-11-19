---
id: 17
title: Engine trait, effects and deterministic context
milestone: M1
status: proposed
depends_on: [1]
size: M
area: [server/crates/engines]
attempts: 0
branch: null
---

## Goal
Invariant 4: engines never write; they return effects (PLAN §13, DOMAIN_MODEL §11). Define in
`mnemo-engines`: the `Engine` trait (all hooks incl. the activity contract `next_items`,
`on_activity_result`, `signals`), `Effect` (`SetState`, `Upsert`, `EmitEvent`, `Schedule`,
`Reveal`), `Signal`, `ItemResult`, `ToolResult`, a read-only `Ctx` trait, and an `EngineRegistry`
mapping built-in ids/versions to implementations.

## Acceptance criteria
- [ ] Default hook implementations return no effects, so an engine implements only what it needs
      — unit test with a no-op engine.
- [ ] `Ctx` exposes read-only `get_entity`, `query(filter)`, `get_state`, `now()`, `random()` and
      `config()` — no method can write (compile-time: trait has no `&mut` store access).
- [ ] A `FakeCtx` test helper (in-memory entities/state, fixed `now`, seeded RNG) makes hooks
      deterministic: two runs with the same seed give identical effects — test.
- [ ] `Effect` and `Signal` serialise to stable JSON (snake_case tags) — tests.
- [ ] `EngineRegistry::resolve(id, version_req)` finds a built-in by semver; unknown → error —
      tests.
- [ ] Engine `config_schema` (JSON Schema) is checked against a binding's `config` by
      `EngineRegistry::validate_config` — test.
- [ ] `just check` passes.

## Notes
Keep the trait object-safe (`Box<dyn Engine>`). Effects reference entities by id; the host (0018)
validates them. If the trait shape differs from DOMAIN_MODEL §11, update that section in the same
change.

## Log
