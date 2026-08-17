#!/usr/bin/env python3
"""Stop-hook for the `rust-porter` agent (wired in its own frontmatter, so
it only applies to that agent — see .claude/agents/rust-porter.md).

Refuses to let the agent finish a turn while `cargo check` is failing, by
returning {"decision": "block", "reason": ...} on stdout, which Claude Code
feeds back to the agent as a reason to keep working.

Always checks stop_hook_active first: if this hook already blocked once
this turn, it must not block again, or a build that's stuck failing would
loop forever. That's the documented safety pattern for Stop/SubagentStop
hooks.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        data = {}

    if data.get("stop_hook_active"):
        return 0

    project_dir = os.environ.get("CLAUDE_PROJECT_DIR", ".")
    if not os.path.exists(os.path.join(project_dir, "Cargo.toml")):
        return 0  # nothing ported yet, nothing to enforce

    try:
        result = subprocess.run(
            ["cargo", "check", "--quiet", "--all-targets"],
            cwd=project_dir,
            capture_output=True,
            text=True,
            timeout=110,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        # Don't block on infrastructure problems (e.g. cargo not on PATH
        # yet during initial setup) — that's not what this hook is for.
        print(f"verify-build hook skipped: {e}", file=sys.stderr)
        return 0

    if result.returncode != 0:
        tail = "\n".join((result.stdout + result.stderr).splitlines()[-60:])
        print(json.dumps({
            "decision": "block",
            "reason": (
                "cargo check is still failing:\n\n"
                f"{tail}\n\n"
                "Fix the compile errors before finishing this module."
            ),
        }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
