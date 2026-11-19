---
name: write-adr
description: Use when a design decision is made or changed (stack, formats, invariants, security, conventions) to record it as an Architecture Decision Record in docs/adr/.
---

# Write an ADR

1. Look at `docs/adr/` for the next number and for ADRs this one relates to or supersedes.
2. Create `docs/adr/NNNN-short-title.md`:

```markdown
# NNNN. <Title>

- Status: accepted | superseded by NNNN
- Date: YYYY-MM-DD

## Context
What forces the decision; constraints; links to PLAN/DOMAIN_MODEL sections.

## Decision
What we do, stated plainly.

## Consequences
What becomes easier, harder, or must now be true. What would make us revisit it.
```

3. If it supersedes an ADR, update that ADR's status line.
4. Update `docs/PLAN.md` / `docs/DOMAIN_MODEL.md` if they describe the old decision.
