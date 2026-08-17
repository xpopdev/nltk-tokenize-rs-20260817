#!/usr/bin/env python3
"""PreToolUse hook, matched to Bash (see .claude/settings.json): stops a
`git commit` from going through while matrix_report.json records failures.

This is a second, deterministic layer on top of the agents' own discipline
— a stuck or over-eager agent shouldn't be *able* to commit a known-broken
port, regardless of what it currently believes about its own progress.

Exit code 2 blocks the command and feeds stderr back to Claude as the
reason, per the Claude Code PreToolUse hook protocol.
"""
from __future__ import annotations

import json
import os
import re
import sys


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    command = data.get("tool_input", {}).get("command", "")
    if not re.search(r"(^|[;&|]\s*)git\s+commit\b", command):
        return 0

    project_dir = os.environ.get("CLAUDE_PROJECT_DIR", ".")
    report_path = os.path.join(project_dir, "matrix_report.json")
    if not os.path.exists(report_path):
        return 0  # nothing recorded yet — don't block early, pre-test commits

    try:
        with open(report_path) as f:
            rows = json.load(f)
    except (json.JSONDecodeError, OSError):
        return 0

    failures = [r for r in rows if r.get("status") != "pass"]
    if failures:
        print(
            f"Blocked: matrix_report.json shows {len(failures)} failing "
            "case(s). Run the verifier agent to fix them (or re-run the "
            "matrix after a fix) before committing.",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
