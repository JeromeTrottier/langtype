# devloop

Local agent dispatcher (docs/DEVELOPMENT.md, ADR 0005).
- `schedule.rs` is pure and fully tested: change scheduling rules there, with tests.
- `outcome.rs` parses headless `claude -p --output-format json`; re-verify field names when the CLI changes.
- Only devloop writes `work/tasks/`; agents report via the `DEVLOOP_RESULT:` line.
