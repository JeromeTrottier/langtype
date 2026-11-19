#!/bin/sh
# PostToolUse hook: format the file that was just edited. Never blocks.
path=$(python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("tool_input",{}).get("file_path",""))')
[ -z "$path" ] && exit 0
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
case "$path" in
  *.rs) rustfmt --edition 2024 --config-path "$root/rustfmt.toml" "$path" >/dev/null 2>&1 ;;
  "$root"/client/*.ts|"$root"/client/*.tsx|"$root"/client/*.css|"$root"/client/*.json|"$root"/client/*.md)
    (cd "$root/client" && pnpm exec prettier --write --log-level silent "$path") >/dev/null 2>&1 ;;
esac
exit 0
