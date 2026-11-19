---
name: implement-task
description: Use when asked to implement a task file from work/tasks (e.g. "/implement-task work/tasks/0007-entity-store.md") or when devloop runs the implementer. Tests first, then code, then all checks, then commit on the task branch.
---

# Implement a task

Input: the absolute path of a task file. You run inside the task's git worktree, on its branch.

1. **Understand.** Read the task file, `CLAUDE.md` (root and in the crates you'll touch), the
   relevant parts of `docs/PLAN.md` and `docs/DOMAIN_MODEL.md`, and the ADRs. Check the log for
   earlier attempts. If the task is ambiguous or conflicts with an invariant or ADR, stop and
   report `blocked` with the question.
2. **Tests first.** Turn every acceptance criterion into a failing test (use the `test-writer`
   subagent for larger tasks). Run them and confirm they fail for the right reason.
3. **Implement** the smallest change that makes them pass. Stay inside the task's scope; note
   follow-ups in your summary instead of doing them.
4. **Check.** Run `just check` until it passes. Fix warnings; don't silence them.
5. **Docs.** If behaviour or design changed, update the docs; if you made a design decision,
   add an ADR (`/write-adr`).
6. **Commit** on the current branch: `git add -A && git commit -m "<type>(<area>): <what> (#<id>)"`.
   Never push, merge or switch branches. Never edit `work/tasks/`.
7. **PR description and validation screenshots** (devloop turns them into the pull request;
   `.devloop-pr/` is git-ignored, don't commit it):
   - Write `.devloop-pr/description.md` with `## Summary` (what and why, 2–4 sentences),
     `## Changes` (bullets by area), `## How it was validated` (commands run and what they
     showed), `## Notes` (follow-ups, limitations). Plain words; no task log, no criteria list
     (devloop adds those).
   - Screenshots into `.devloop-pr/screenshots/` with `just screenshot`:
     - **always** the tests that prove the task, e.g.
       `just screenshot --run "cargo test -p mnemo-store" --name store-tests`;
     - **UI changes:** every page you changed, e.g.
       `just screenshot --page /settings --name settings --dark` (starts its own server and
       dashboard on free ports).
     A failing `--run` means the work isn't done.
8. **Self-review** the diff (`git diff <base>...HEAD`) against the acceptance criteria and the
   invariants in `CLAUDE.md`.
9. Final message: what changed, how it's tested, any follow-ups, then
   `DEVLOOP_RESULT: {"status": "agent_review", "summary": "<one line>"}`
   (or `"blocked"` with the reason).
