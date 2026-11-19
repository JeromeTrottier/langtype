#!/usr/bin/env python3
"""PreToolUse hook: block edits to protected paths (docs/DEVELOPMENT.md §7).

- LICENSE, schemas/ and docs/FILTERS.md: only when the branch also adds an ADR in docs/adr/.
- Applied migrations (files that already exist on the base branch): never.
- work/tasks/: only the planner (devloop) may write there.
Exit code 2 blocks the tool call and shows the message to Claude.
"""
import json
import os
import subprocess
import sys

data = json.load(sys.stdin)
tool_input = data.get("tool_input", {})
path = tool_input.get("file_path") or tool_input.get("notebook_path") or ""
if not path:
    sys.exit(0)

root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip()
rel = os.path.relpath(os.path.abspath(path), root) if root else path


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, cwd=root)


def branch_adds_adr():
    base = os.environ.get("DEVLOOP_BASE", "main")
    changed = git("diff", "--name-only", f"{base}...HEAD", "--", "docs/adr").stdout.split()
    untracked = git("ls-files", "--others", "--exclude-standard", "docs/adr").stdout.split()
    staged = git("diff", "--name-only", "--cached", "--", "docs/adr").stdout.split()
    return bool(changed or untracked or staged)


def block(msg):
    print(msg, file=sys.stderr)
    sys.exit(2)


if rel.startswith("work/tasks/") and os.environ.get("DEVLOOP") == "1" and os.environ.get("DEVLOOP_ROLE") != "planner":
    block("work/tasks/ is managed by devloop; report results with the DEVLOOP_RESULT line instead.")

if rel.startswith("server/crates/store/migrations/"):
    base = os.environ.get("DEVLOOP_BASE", "main")
    on_base = git("cat-file", "-e", f"{base}:{rel}").returncode == 0
    if on_base:
        block(f"{rel} is an applied migration; add a new migration file instead.")

if rel == "LICENSE" or rel.startswith("schemas/") or rel == "docs/FILTERS.md":
    if not branch_adds_adr():
        block(f"{rel} is a contract file; add an ADR in docs/adr/ (use /write-adr) before changing it.")

sys.exit(0)
