#!/usr/bin/env bash
# PostToolUse hook (matched to Write|Edit|MultiEdit in .claude/settings.json):
# auto-formats whatever file Claude just touched, so style is consistent
# without spending a turn asking Claude to run a formatter itself.
#
# Best-effort by design: a missing formatter binary should never break the
# agentic loop, so this never exits non-zero.
set -uo pipefail

INPUT="$(cat)"
FILE_PATH="$(printf '%s' "$INPUT" | python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get("tool_input", {}).get("file_path", ""))
except Exception:
    print("")
' 2>/dev/null)"

[ -z "$FILE_PATH" ] && exit 0
[ -f "$FILE_PATH" ] || exit 0

case "$FILE_PATH" in
  *.rs)
    command -v rustfmt >/dev/null 2>&1 && rustfmt --edition 2021 "$FILE_PATH" 2>/dev/null
    ;;
  *.py)
    command -v black >/dev/null 2>&1 && black --quiet "$FILE_PATH" 2>/dev/null
    ;;
esac

exit 0
