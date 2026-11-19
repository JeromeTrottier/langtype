---
id: 9
title: Check `requires` and resolve `extends`
milestone: M1
status: proposed
depends_on: [8]
size: M
area: [server/crates/kits, server/crates/cli]
attempts: 0
branch: null
---

## Goal
Packaging fields (PLAN §11 Dependencies, DOMAIN_MODEL §4 invariants): `requires` must be satisfiable
by the platform, and `extends` names one parent kit merged at install time (depth ≤ 1, merged
result valid). Add a `Platform { version, engine_api, engines: map<id, version> }` input to
validation and an `extends` resolver over a kit lookup.

## Acceptance criteria
- [ ] `requires.platform` that the given platform version doesn't satisfy is reported (semver
      ranges like `>=0.1 <1.0`) — test.
- [ ] `requires.engine-api` and `requires.engines.<id>` checked the same way; an unknown engine id
      is reported — tests.
- [ ] `extends: "@a/parent"` merges the parent: child entity types/fields/events/roles/skills
      added or overriding by key — test with two fixture kits under `kits/_test/extends/`.
- [ ] A parent that itself `extends` something is rejected (depth ≤ 1) — test.
- [ ] A missing parent, or a merged result that fails validation, is reported — tests.
- [ ] `kit validate` uses the current build's platform (version from `CARGO_PKG_VERSION`, engine
      API 1, built-in engines declared in one constant list) — CLI test.
- [ ] `just check` passes.

## Notes
The built-in engine list for M1 is `counters` (0019); put it where both `kits` and `engines` can
use it without a dependency cycle (e.g. a const in `core`), and say so in the PR. Library
resolution (`requires.libraries`) stays a declaration check until M3.

## Log
