#!/usr/bin/env bash
# Notification hook: pings you when Claude needs input (e.g. the PLAN.md
# approval gate) instead of you needing to keep the terminal in view.
#
# Uses termux-toast on Termux (needs `pkg install termux-api` + the
# Termux:API companion app from F-Droid/Play Store), falls back to
# notify-send on Linux desktops, osascript on macOS, and silently no-ops
# anywhere else rather than failing the hook.
set -uo pipefail

MSG="Claude needs your input"

if command -v termux-toast >/dev/null 2>&1; then
  termux-toast "$MSG" >/dev/null 2>&1
elif command -v notify-send >/dev/null 2>&1; then
  notify-send "Claude Code" "$MSG" >/dev/null 2>&1
elif command -v osascript >/dev/null 2>&1; then
  osascript -e "display notification \"$MSG\" with title \"Claude Code\"" >/dev/null 2>&1
fi

exit 0
