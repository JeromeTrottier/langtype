#!/bin/sh
# Stop hook (devloop implementer/fixer only): don't hand off while fast checks fail.
# Exit 2 tells Claude to keep working; `stop_hook_active` prevents loops.
[ "$DEVLOOP" = "1" ] || exit 0
case "$DEVLOOP_ROLE" in implementer|fixer) ;; *) exit 0 ;; esac
active=$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("stop_hook_active", False))')
[ "$active" = "True" ] && exit 0
root=$(git rev-parse --show-toplevel) || exit 0
if ! out=$(cd "$root" && just check-fast 2>&1); then
  printf '%s\n' "just check-fast failed; fix it before finishing:" "$(printf '%s' "$out" | tail -40)" >&2
  exit 2
fi
exit 0
