---
name: milestone-status
description: Use when asked how development is going, what's blocked, or for a milestone progress report. Summarises work/tasks and devloop state.
---

# Milestone status

1. Run `cargo run -q -p devloop -- status` (or read `work/tasks/*.md` and `.devloop/state.json`).
2. Report, briefly:
   - milestone and progress (done / total), what's running now;
   - tasks **awaiting your review** and **blocked** (with the reason from the log) — these need the human;
   - proposed tasks waiting for approval;
   - risks: tasks with several fix attempts, long-running ones, today's usage vs budget.
3. End with the one or two most useful next actions for the human.
