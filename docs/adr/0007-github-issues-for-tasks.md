# 0007. GitHub issues as the human interface to the task queue

- Status: accepted
- Date: 2026-11-19
- Amends: 0005, 0006

## Context
Approving, editing and answering tasks required the terminal (`devloop approve`, editing task
files). The human should be able to steer development from GitHub alone, including on a phone.

## Decision
- devloop mirrors each task to a GitHub issue: body from the task (goal, criteria, notes,
  dependencies as issue links), status as a `devloop: …` label, milestone from the task.
- devloop reads back only the owner's actions: `approved` label → ready; edits before work starts
  → task text; comments → notes (and unblock a blocked task); closing → cancelled.
- PRs say `Closes #N`, so merging closes the issue.
- Task files stay the agents' working copy; devloop is the only writer of files and issues.
  A new `cancelled` status marks dropped tasks.

## Consequences
No terminal needed for day-to-day steering. Task descriptions are public, like the code. Issue
state lags task state by up to one devloop cycle.
