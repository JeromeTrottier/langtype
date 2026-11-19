---
name: sync-docs
description: Use after changes that may have made docs/PLAN.md, docs/DOMAIN_MODEL.md, docs/FILTERS.md or CLAUDE.md files inaccurate, to bring them back in line with the code.
---

# Sync docs with code

1. Find what changed: `git diff main...HEAD --stat` (or the range you're given).
2. For each changed area, read the matching doc sections and the code.
3. Fix statements that are now false (names, fields, flows, commands). Don't add speculation or
   plans; keep the docs' style (short sections, tables, plain words).
4. If a change contradicts a design decision rather than refining it, don't silently rewrite the
   doc: write an ADR or report it as a question.
5. Summarise what you changed and why.
