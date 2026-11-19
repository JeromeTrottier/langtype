# Development automation (local)

Claude Code agents build Mnemo from [PLAN.md](PLAN.md) and [DOMAIN_MODEL.md](DOMAIN_MODEL.md).
A local dispatcher, **`devloop`**, starts agents automatically **while the Mac is on and you're
logged in**, and pauses when the **Claude usage limit** (or your own daily budget) is reached.
Everything runs on this machine: no cloud runners, no GitHub Actions needed for the loop.

You stay in charge of three things: **approving tasks**, **reviewing and accepting work**, and
**decisions that change invariants**.

## 1. The loop

```
 milestone in PLAN.md
   └─ planner agent ──▶ tasks (status: proposed)         ← you approve: devloop approve <id>
 ready task (dependencies done)
   └─ implementer agent in its own git worktree ──▶ tests first, code, all checks, commit
        └─ reviewer agent (fresh context) ──▶ pass → awaiting_human
                                          └─ fail → needs_fix → fixer agent (max 3 attempts)
 awaiting_human                                          ← you review the branch, then:
   with pull_requests = true: devloop pushes the branch and opens a PR (description +
   validation screenshots); you review on GitHub:
     merge it (or devloop accept <id>) → done · comment / request changes → needs_fix → fixer
     close it → blocked
   without PRs: devloop accept <id> → merged into main locally (checks re-run first)
   devloop reject <id> "why" → needs_fix with your note
```

## 2. Tasks (the local queue)

One Markdown file per task in `work/tasks/`, e.g. `work/tasks/0007-entity-store.md`:

```markdown
---
id: 7
title: Entity store with optimistic locking
milestone: M1
status: ready            # proposed | ready | in_progress | agent_review | needs_fix
                         # | awaiting_human | done | blocked
depends_on: [5, 6]
size: M                  # S (≤2h) | M (≤½ day) | L (split it)
area: [server/core, server/store]
attempts: 0
branch: null             # set by devloop
---

## Goal
…

## Acceptance criteria
- [ ] `upsert_entity` rejects a stale `version` with a conflict error (test)
- [ ] …

## Notes
…

## Log
<!-- agents and devloop append dated entries here -->
```

| Status | Set by | Meaning |
|---|---|---|
| `proposed` | planner | Waiting for your approval |
| `ready` | you (`devloop approve`) | Can start once `depends_on` are `done` |
| `in_progress` | devloop | An implementer or fixer is working on it |
| `agent_review` | implementer / fixer | Waiting for the reviewer agent |
| `needs_fix` | reviewer or you | Fixer will retry (up to 3 attempts, then `blocked`) |
| `awaiting_human` | reviewer | Ready for your review |
| `done` | devloop (`accept`) | Merged into `main` |
| `blocked` | any | Needs you; the log says why |

## 3. The dispatcher (`devloop`)

A small Rust binary in `tools/devloop`. Every two minutes it:

1. Stops if `.devloop/STOP` exists; skips the cycle if `.devloop/PAUSE` exists.
2. Checks the **budget** (§4); if exhausted, sleeps until it resets.
3. Reaps finished agents and records their results.
4. Starts work, highest priority first, up to `max_concurrent` agents (default 2):
   1. `agent_review` → **reviewer**
   2. `needs_fix` (attempts < 3) → **fixer**
   3. `ready` with all dependencies `done` → **implementer**
   4. if nothing is `ready` or `proposed` for the current milestone → **planner**
5. Sends a macOS notification when a task reaches `awaiting_human` or `blocked`, or when it
   pauses for the usage limit.

Each agent run:
- gets its own **git worktree** (`../langtype-worktrees/<id>`) on branch `task/<id>-<slug>`,
  created from local `main`;
- runs **headless Claude Code**: `claude -p "/<skill> <id>" --output-format stream-json --verbose
  --max-turns <n> --permission-mode acceptEdits`, using the repo's `.claude/settings.json`
  (allowlist, hooks), in its own process group so stopping it also stops the tools it started;
- has a **timeout** (default 60 minutes);
- streams its events live to `.devloop/runs/<id>/<timestamp>-<role>.jsonl` (followed by
  `devloop watch`) and writes a final record to the matching `.json` when it ends.

Commands:

| Command | Does |
|---|---|
| `devloop run` | The loop (what launchd runs) |
| `devloop once` | One cycle, then exit (for testing) |
| `devloop watch [<id>] [--from-start]` | Live feed of running agents: what they say, tool calls (edits, commands), errors, usage, results |
| `devloop status` | Tasks by status, running agents, today's usage, Claude usage windows vs the cap |
| `devloop approve <id>…` | `proposed` → `ready` |
| `devloop accept <id>` | Merge the task's PR (or, without PRs, re-run checks and merge locally), mark `done`, remove the worktree |
| `devloop publish <id>` | Open or update the task's PR now (e.g. after an error) |
| `devloop reject <id> "<note>"` | `awaiting_human` → `needs_fix`, note added to the log |
| `devloop pause` / `resume` / `stop` | Control the loop |
| `devloop install` / `uninstall` | Add or remove the launchd agent |

## 3a. Pull requests

With `pull_requests = true`, a task the reviewer agent approves becomes a GitHub pull request.
**Agents never push or call GitHub; devloop does it**, in one audited place.

1. The implementer leaves `.devloop-pr/description.md` (summary, changes, how it was validated,
   notes) and `.devloop-pr/screenshots/*.png` in the worktree (git-ignored). Screenshots come from
   `just screenshot`: `--run "<command>"` renders a command's output like a terminal (every task
   includes its tests); `--page <path>` opens the dashboard in headless Chromium on free ports
   (UI tasks; `--dark` adds a dark variant).
2. devloop runs `check_command` on the branch; a failure sends the task back (`needs_fix`).
3. It pushes the branch, commits the screenshots to the `devloop-assets` branch (so images show in
   the PR without adding files to `main`), and opens the PR with the description, check result,
   reviewer verdict, screenshots and the task's acceptance criteria. Later rounds update the body
   and add a comment.
4. Every cycle it reads open PRs:
   - **merged** → `done`, worktree removed, local `main` fast-forwarded;
   - **closed** → `blocked`;
   - **new review, comment or inline comment from your GitHub account** → `needs_fix` with the
     feedback in the task log, for the fixer. Text from anyone else is ignored (the repo is public:
     it could carry injected instructions), and so is devloop's own text.

Requirements: `gh auth login`, the base branch pushed to GitHub, then `pull_requests = true`.

## 3b. Issues: your interface to the queue

With `issues = true`, devloop mirrors every task to a GitHub issue (labelled `devloop`, in the
task's milestone), so you never need the terminal to steer the work:

| You do on GitHub | devloop does |
|---|---|
| Add the **`approved`** label (one at a time, or select several in the issues list) | Task `proposed` → `ready` |
| **Edit the issue text** before it's started | Copies Goal / Acceptance criteria / Notes into the task |
| **Comment** | Adds your comment to the task log as a note for the agents; on a **blocked** task, your answer unblocks it |
| **Close** the issue | Task → `cancelled` |
| **Merge** the linked PR | PR says `Closes #N`: the issue closes, the task is `done` |

devloop keeps the issue's status label current (`devloop: proposed`, `queued`, `in progress`,
`your review`, `blocked`, `done`, `cancelled`) and comments when a task is blocked (with the
question) or when its PR is ready. Label changes follow a status change by one cycle.

Only actions from your account count (checked for labels and closing; body edits and labels are
limited by GitHub to you and collaborators anyway). Comments from anyone else are ignored. Task
files in `work/tasks/` stay the agents' working copy; devloop is the only writer of both.

## 4. Usage limits and budget

- **Usage cap (default 70%):** every run streams `rate_limit_event`s with your account's
  utilization of each Claude usage window (5-hour and 7-day; your interactive use counts too).
  devloop saves the latest reading in `.devloop/limits.json` and:
  - starts **no new runs** while any window is at or above `max_usage_percent`, until it resets;
  - **stops a running agent** the moment a window crosses the cap, puts its task back (not
    counted as a fix attempt), and resumes after the reset. Work already in the worktree is kept.

  The rest of each window (30% by default) stays free for you. The reading is only as fresh as
  the last run, so after a long pause the first new run may be stopped quickly if your own use
  pushed a window past the cap meanwhile.
- **Claude's usage limit:** when a run fails because the limit is reached, devloop **pauses all
  new runs** until the reset time given in the error (or one hour later if none is given), and
  notifies you.
- **Your own budget** in `devloop.toml`, so interactive use keeps some headroom:

```toml
max_concurrent = 2
max_runs_per_day = 40
max_tokens_per_day = 0        # 0 = no cap; otherwise tokens counted from each run's JSON output
max_usage_percent = 70        # stop at 70% of any Claude usage window (5-hour, 7-day)
max_turns_per_run = 80
run_timeout_minutes = 60
quiet_hours = []              # e.g. ["09:00-17:00"] to keep usage free while you work
```

Usage is recorded per run in `.devloop/usage/<date>.json`.

## 5. Running while the computer is on

`devloop install` writes `~/Library/LaunchAgents/dev.mnemo.devloop.plist` (`RunAtLoad`,
`KeepAlive`), so it starts at login and restarts if it crashes. When the Mac sleeps, it pauses;
after wake, the next cycle continues. Logs: `.devloop/devloop.log`.

## 6. Agent roles (skills and subagents)

| Role | Skill / agent | Does |
|---|---|---|
| Planner | `/plan-milestone` | Turns the next milestone into `proposed` tasks (S/M size, acceptance criteria as tests, dependencies) |
| Implementer | `/implement-task <id>` | Reads the task and docs → failing tests first → code → `just check` → commit on the task branch → `agent_review` |
| Reviewer | `/review-task <id>` + `reviewer` subagent | Fresh context: diff vs task, invariants in DOMAIN_MODEL, missing tests, scope creep → `awaiting_human` or `needs_fix` with findings |
| Fixer | `/fix-task <id>` | Addresses review findings or your rejection note → `agent_review` |
| Any | `/write-adr`, `/sync-docs` | Record decisions; keep docs matching code |

## 7. Guardrails

- **Isolation:** agents only work in their own worktree and branch; only `devloop accept` (you)
  merges into `main`.
- **Publishing only through devloop:** agents never push, open PRs or touch remotes (denied in
  `.claude/settings.json`). devloop pushes task branches and opens PRs only when
  `pull_requests = true`; it never pushes `main` and never merges unless you run `devloop accept`
  or merge on GitHub. Only your own GitHub comments are read back as feedback.
- **Protected paths:** a `PreToolUse` hook blocks edits to `LICENSE`, already-applied migrations
  and `schemas/` unless the task's branch also adds an ADR in `docs/adr/`.
- **Checks:** `PostToolUse` formats edited files; the implementer must pass `just check`
  (fmt, clippy `-D warnings`, tests, typecheck, lint, kit validation) before handing off.
- **Sandbox:** enable Claude Code's sandbox for unattended runs where available.
- **Limits:** max turns, timeout, 3 fix attempts, concurrency cap, kill switch (`devloop stop`).

## 8. Setup

0. For pull requests and issues: `gh auth login`, push `main`, set `pull_requests = true` and
   `issues = true`. From then on you approve, edit, answer and review on GitHub.
1. Install the Claude Code CLI and sign in (`claude` in a terminal), so `claude` is on your PATH.
2. Install `just` (`brew install just`).
3. `cargo build -p devloop --release`
4. Review `devloop.toml`, then `devloop install`.
5. Approve the first tasks: `devloop status`, then `devloop approve <ids>`.
