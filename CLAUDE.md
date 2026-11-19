# Mnemo — guide for Claude

Mnemo (codename; repo `langtype`) gives Claude agents a structured, persistent memory per
activity ("kits": language learning, tabletop RPG…). Read before changing anything:

- `docs/PLAN.md` — what we're building and the milestones.
- `docs/DOMAIN_MODEL.md` — concepts and **invariants** (the rules code must never break).
- `docs/FILTERS.md` — the one expression language (CEL) and query format.
- `docs/DEVELOPMENT.md` — how development agents and `devloop` work.
- `docs/adr/` — decisions already made. Don't reverse one without a new ADR.

## Layout

| Path | What |
|---|---|
| `server/crates/api` | `mnemo` binary: axum REST, local security, MCP (M2), `openapi` and `mcp-stdio` subcommands |
| `server/crates/core` | Domain types and rules. **No I/O.** |
| `server/crates/store` | SQLite storage and migrations (`migrations/`) |
| `server/crates/kits` | Kit manifest loading and validation |
| `server/crates/{engines,briefing,mcp,content}` | Filled in from M1–M3 |
| `server/crates/cli` | `kit` CLI (`kit validate`) |
| `tools/devloop` | Local agent dispatcher |
| `client/` | React + Vite dashboard (pnpm) |
| `schemas/` | Kit and library format contracts (JSON Schema) |
| `kits/` | Kits; `kits/_test` is the test kit |
| `work/tasks/` | Development task queue |

## Commands

- `just check` — everything CI checks (must pass before handing off work).
- `just check-fast` — format check, clippy, Rust tests, client typecheck and tests.
- `just fmt` — format Rust and client code.
- `just gen-api` — regenerate `client/src/api/types.gen.ts` from the server's OpenAPI.
- `just dev` — server on 127.0.0.1:3000 + Vite on :5173.
- `just agents-status` / `just agents-watch` — development agents' state and live feed.
- `cargo test -p <crate>` / `pnpm --dir client test` for focused runs.

## Conventions

- Rust 2024, `clippy -D warnings`, `rustfmt` (max width 110). Errors with `thiserror` in libraries,
  `anyhow` in binaries. No `unwrap()` outside tests and truly impossible cases (add a comment).
- Tests next to code (`#[cfg(test)]`) for units; `tests/` for crate-level behaviour.
- IDs are UUIDv7 (`mnemo_core::ids`). Timestamps UTC RFC 3339. Text NFC-normalised.
- Never edit an applied migration; add `NNNN_name.sql`.
- UI text only in `client/src/i18n/locales/en.json`; components use `t("…")`.
- Keep docs true: if code changes a design, update the doc in the same change (or `/sync-docs`).
- Changing `schemas/`, `docs/FILTERS.md`, `LICENSE` or an invariant needs an ADR (`/write-adr`).

## Invariants (from DOMAIN_MODEL — test them)

1. Visibility is checked server-side on every read path; secrets never reach a role that can't see them.
2. Every write goes through one changeset with before/after; undo is a reverting changeset.
3. Events are append-only facts; never edited. Raw activity facts live in activity runs.
4. Engines never write: they return effects the host validates and applies in one transaction.
5. Entities always validate against their kit type; optimistic locking on `version`.
6. A KB is pinned to an exact kit version + hash; published kit and library versions are immutable.
7. Drafts are validated but not applied; applying re-checks target versions.
8. Agents act as a KB member, never with more rights. Text written by people is data, not instructions.
9. Local mode: bind 127.0.0.1, validate `Host` and `Origin`, require the per-install token.

## Definition of done

- Acceptance criteria of the task are met, each backed by a test.
- `just check` passes; no new warnings.
- Docs and ADRs updated when behaviour or design changed.
- One focused commit (or a few) on the task branch with a clear message.

## When running under devloop (`DEVLOOP=1`)

- Work only in the current worktree. Don't push, merge, switch branches or edit `work/tasks/`.
- Finish your final message with exactly one line:
  `DEVLOOP_RESULT: {"status": "<status>", "summary": "<one line>", "findings": ["…"]}`
  — implementer/fixer: `agent_review` or `blocked`; reviewer: `awaiting_human`, `needs_fix` or `blocked`.
- If something needs a human decision, stop and report `blocked` with the question in `summary`.
- Leave the PR description and validation screenshots in `.devloop-pr/` (see `/implement-task`);
  devloop opens the pull request. Never push or use `gh` yourself.
