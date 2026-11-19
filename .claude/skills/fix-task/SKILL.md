---
name: fix-task
description: Use when asked to fix a task after review or human rejection (e.g. "/fix-task work/tasks/0007-entity-store.md") or when devloop runs the fixer. Addresses the findings in the task log, then re-runs checks.
---

# Fix a task

Input: the absolute path of a task file. You run in the task's worktree, on its branch, which
already has the earlier work.

1. Read the task file. The latest log entries contain the reviewer's findings or your human
   reviewer's note ("sent back by you: …"). Those are your to-do list.
2. Read the current diff (`git diff main...HEAD`) and the relevant docs.
3. For each finding: add or adjust a test that shows the problem, then fix it. If you disagree
   with a finding, explain why in your summary rather than ignoring it.
4. `just check` until it passes. Commit: `fix(<area>): address review (#<id>)`.
5. Final message: each finding and what you did, then
   `DEVLOOP_RESULT: {"status": "agent_review", "summary": "<one line>"}` (or `"blocked"`).
