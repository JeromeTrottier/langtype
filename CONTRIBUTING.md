# Contributing

Thanks for helping! This project is largely developed by Claude Code agents under human review
(see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)); human contributions follow the same rules.

1. Read [CLAUDE.md](CLAUDE.md): layout, commands, conventions and the invariants.
2. Work on a branch; keep changes focused on one task.
3. Write tests first for new behaviour; invariants get property tests.
4. Run `just check` before opening a pull request.
5. Design decisions go in an ADR (`docs/adr/`); changes to `schemas/` or `docs/FILTERS.md` require one.
6. Keep the docs true: update PLAN / DOMAIN_MODEL when behaviour changes.

By contributing you agree that your contributions are licensed under Apache-2.0 (code) and
CC BY 4.0 (kit content we publish).
