# Mnemo

*Codename. Repository: `langtype`.*

Structured, persistent memory for Claude agents. Start a knowledge base for any long-running
activity — learning a language, running a tabletop campaign, coaching — and Claude gets the right
state, rules and history at the start of every session, then records what happened. You own the
data and can see and edit it in a dashboard.

**Status:** early development (milestone M0: foundations). See [docs/PLAN.md](docs/PLAN.md).

## Documents

- [docs/PLAN.md](docs/PLAN.md) — vision, design and milestones
- [docs/DOMAIN_MODEL.md](docs/DOMAIN_MODEL.md) — concepts and invariants
- [docs/FILTERS.md](docs/FILTERS.md) — expressions (CEL) and queries
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — how Claude agents develop this project (devloop)
- [docs/adr/](docs/adr/) — decisions

## Getting started

Requirements: Rust (stable), Node 24 + pnpm, [`just`](https://github.com/casey/just).

```bash
pnpm --dir client install
just install-browser   # headless Chromium for validation screenshots
just dev        # server on http://127.0.0.1:3000, dashboard on http://localhost:5173
just check      # everything CI checks
```

The server only listens on 127.0.0.1 and requires a per-install token (created in `data/local_token`
on first run) for everything except `/api/v1/health`.

Validate a kit:

```bash
cargo run -p mnemo-cli --bin kit -- validate kits/_test
```

## Licence

Code: [Apache-2.0](LICENSE). Content in official kits: CC BY 4.0; libraries keep their sources'
licences (see [ADR 0003](docs/adr/0003-licensing.md)).
