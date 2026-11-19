# 0006. Pull requests opened by devloop, with validation screenshots

- Status: accepted
- Date: 2026-11-18
- Amends: 0005 (work is reviewed on GitHub instead of only locally)

## Context
Finished tasks should arrive as pull requests with a clear description and visual proof of
validation. ADR 0005 keeps agents away from remotes, and the repository is public.

## Decision
- Agents write `.devloop-pr/description.md` and validation screenshots (`just screenshot`,
  Playwright: page screenshots for UI, rendered terminal output for tests). They still never push
  or call GitHub.
- devloop (when `pull_requests = true`) re-runs the checks, pushes the task branch, stores the
  screenshots on an orphan `devloop-assets` branch, and opens or updates the PR with `gh`.
- devloop follows PRs: merged → done; closed → blocked; feedback **from the repository owner's
  account only** → needs_fix for the fixer. Its own text carries a marker and is never re-read.

## Consequences
GitHub becomes the review surface. Screenshots don't bloat `main`. Review comments from others
on the public repo can't steer agents. Requires `gh` authentication and the base branch on GitHub.
