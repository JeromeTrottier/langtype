# 0005. Local development automation with devloop

- Status: accepted
- Date: 2026-11-18

## Context
Claude agents should build most of the product, automatically while the Mac is on, without
exceeding usage limits, and without cloud runners (docs/DEVELOPMENT.md).

## Decision
- A Rust dispatcher (`tools/devloop`) runs headless Claude Code on a file-based queue
  (`work/tasks/`), each task in its own git worktree and branch; started by launchd at login.
- Roles: planner, implementer, reviewer, fixer — project skills in `.claude/skills/`.
- Agents report with a final `DEVLOOP_RESULT:` line; only devloop edits task files.
- Usage-limit errors pause new runs until reset. A **usage cap** (default 70% of any Claude usage
  window, read from the run's `rate_limit_event`s) stops new runs and running agents until the
  window resets. A daily budget and quiet hours are also configurable.
- Humans approve proposed tasks and accept finished work (`devloop accept` merges into `main`).
  Agents never push, merge or switch branches (settings deny-list + hooks).

## Consequences
The quality of the loop depends on checks (`just check`, hooks, reviewer). Headless CLI flags and
output format must be re-verified when Claude Code updates.
