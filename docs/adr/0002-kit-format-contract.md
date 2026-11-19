# 0002. Kit format contract: JSON Schema + CEL + one filter format

- Status: accepted
- Date: 2026-11-18

## Context
Kits are written by us, by users and by Claude, and read by the runtime, validator, MCP tools
and dashboard. Ad-hoc syntaxes had crept into the plan (PLAN: Expressions and queries).

## Decision
- `schemas/kit.schema.json` and `schemas/library.schema.json` are the contract; the validator
  embeds them and adds reference checks (types, roles, engines, activities, skills, starters,
  libraries).
- All expressions are CEL; all queries use the filter format in `docs/FILTERS.md`.
- `kits/_test` (valid) and `kits/_test/invalid` are fixtures every change must keep passing.

## Consequences
Changing the schemas or FILTERS.md needs an ADR (enforced by a hook). CEL type-checking is added
in M1.
