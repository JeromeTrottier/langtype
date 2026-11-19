# 0003. Licensing

- Status: accepted
- Date: 2026-11-18

## Context
Open source with optional paid hosting later. Seed data comes from sources with their own terms.

## Decision
- All code: Apache-2.0.
- Content we write for official kits (skills, starters, docs): CC BY 4.0.
- Libraries keep their sources' licences: language libraries CC BY-SA 4.0 (Wiktionary,
  FrequencyWords; Tatoeba CC BY 2.0 FR), `srd-5e` CC BY 4.0; attribution shown in the app.

## Consequences
Every library manifest must declare licence, sources and attribution (schema-enforced).
