# core

Domain types and rules from docs/DOMAIN_MODEL.md. **No I/O**: no database, HTTP, MCP, filesystem.
- New concepts get a module; keep types serialisable with serde.
- Rules that protect invariants (visibility, changesets, drafts, effects) live here and get the most
  tests (property tests with `proptest` for invariants).
