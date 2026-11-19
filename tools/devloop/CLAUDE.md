# devloop

Local agent dispatcher (docs/DEVELOPMENT.md, ADR 0005).
- `schedule.rs` is pure and fully tested: change scheduling rules there, with tests.
- `outcome.rs` parses headless `claude -p --output-format stream-json --verbose`: the final `result`
  event and `rate_limit_event`s (`rate_limit_info.unifiedWindows.<window>.{utilization,resetsAt,status}`)
  used for the usage cap. Re-verify field names when the CLI changes.
- Only devloop writes `work/tasks/`; agents report via the `DEVLOOP_RESULT:` line.
