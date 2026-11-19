---
name: review-task
description: Use when asked to review a task's branch (e.g. "/review-task work/tasks/0007-entity-store.md") or when devloop runs the reviewer. Reviews the diff with fresh eyes against the task, invariants and conventions; does not change code.
---

# Review a task

Input: the absolute path of a task file. You run in the task's worktree. **Don't edit files**;
only read, run checks and report.

1. Read the task (goal, acceptance criteria, log) and `CLAUDE.md`.
2. Get the diff: `git diff main...HEAD` (and `git log main..HEAD --oneline`).
3. Use the `reviewer` subagent with the task path and the diff for an independent review.
4. Run `just check` yourself. Any failure is a finding.
5. Decide:
   - **awaiting_human** — every acceptance criterion is met and tested, checks pass, no
     invariant is at risk, scope is respected. Minor nits go in `findings` as "nit:".
   - **needs_fix** — list each concrete problem as a finding: file/line, what's wrong, what
     "fixed" looks like.
   - **blocked** — the task itself is wrong or needs a human decision.
6. Final message: the review, then
   `DEVLOOP_RESULT: {"status": "<awaiting_human|needs_fix|blocked>", "summary": "<one line>", "findings": ["…"]}`
