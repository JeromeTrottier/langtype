---
id: 19
title: Minimal `counters` engine for the test kit
milestone: M1
status: proposed
depends_on: [17, 9]
size: S
area: [server/crates/engines]
attempts: 0
branch: null
---

## Goal
The test kit binds `counters` and declares a `bump` activity, so M1 needs one real engine that
exercises the activity contract end to end. Implement a minimal built-in `counters` engine
(PLAN §13: bounded numeric fields with history): config `track: [type.field]`, `next_items` for an
activity, `on_activity_result` turning results into effects, and a `signals` hook.

## Acceptance criteria
- [ ] `config_schema` accepts `{ track: ["counter.value"] }` and rejects a non-list — test.
- [ ] `next_items` returns visible entities matching the activity's `map` filter via `Ctx::query`,
      respecting its limit — test with `FakeCtx`.
- [ ] `on_activity_result` with `{ entity_id, raw: { delta: 3 } }` returns an `Upsert` of the
      tracked field (+3) and an `EmitEvent` `counter_changed { delta: 3 }` — test.
- [ ] Optional bounds (`min`/`max` in config) clamp the value — test.
- [ ] `signals` returns one signal "N counters below target" when any tracked value is under a
      configured target, opening the `bump` activity; none otherwise — test.
- [ ] Registered in `EngineRegistry` as `counters@1.0.0`, matching the built-in list from 0009
      — test.
- [ ] `just check` passes.

## Notes
Keep it small and generic; the full `counters` engine (HP, spell slots, history views) is M4 and
can grow from this. Labels in signals are kit-provided or neutral ("{n} below target"), never
English product words baked into the engine beyond a default.

## Log
