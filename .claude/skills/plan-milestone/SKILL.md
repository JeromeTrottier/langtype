---
name: plan-milestone
description: Use when asked to plan a milestone (e.g. "/plan-milestone M1") or when devloop runs the planner. Breaks the milestone from docs/PLAN.md into small, testable task files in work/tasks/ with status `proposed`.
---

# Plan a milestone

Input: a milestone id (e.g. `M1`). You run in the main checkout; **only create files in
`work/tasks/`** — change nothing else.

1. Read `docs/PLAN.md` (the milestone's entry in "Milestones" and every section it relies on),
   `docs/DOMAIN_MODEL.md`, `docs/FILTERS.md`, `CLAUDE.md`, the ADRs, and existing `work/tasks/`
   (don't duplicate; continue numbering from the highest id).
2. List the milestone's deliverables, then split them into tasks:
   - each **S (≤2h) or M (≤½ day)**; split anything bigger;
   - each with a clear **goal**, the **area** (crates/folders), and **acceptance criteria written
     as checkable tests** ("`upsert_entity` with a stale version returns `Conflict` — unit test");
   - **dependencies** (`depends_on`) so tasks can run in parallel safely: two tasks that edit
     the same files must depend on each other;
   - foundations first (types, schema, traits), then features, then integration.
3. Write one file per task: `work/tasks/NNNN-short-slug.md` (4-digit id), following
   `work/tasks/_template.md`, with `status: proposed`, `attempts: 0`, `branch: null`.
   **Always quote the title** (`title: "Activity runs: start, complete"`): titles often contain
   colons, which break the YAML frontmatter otherwise.
4. Add a final task "Milestone <id> integration check" that depends on all others and verifies
   the milestone's definition in PLAN.md end to end.
5. Check your files: `cargo run -q -p devloop -- status` must list every new task and print no
   `unreadable:` line. Fix any file it reports.
6. Final message: a table of the tasks (id, title, size, depends on), then
   `DEVLOOP_RESULT: {"status": "proposed", "summary": "<n> tasks proposed for <milestone>"}`
