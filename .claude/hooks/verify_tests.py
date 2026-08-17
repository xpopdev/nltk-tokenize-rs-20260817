#!/usr/bin/env python3
"""Stop-hook for the `verifier` agent (wired in its own frontmatter — see
.claude/agents/verifier.md). Refuses to let it finish a turn while
matrix_report.json still records failures.

Deliberately doesn't re-run the matrix itself (that can be slow, especially
on a constrained device) — it trusts matrix_report.json to be current,
which is the verifier agent's own job to keep true as it iterates.

Checks stop_hook_active first so a genuinely stuck run can still stop and
report back (per the verifier agent's own MAX_ITERATIONS logic) instead of
being blocked forever by this hook.
"""
from __future__ import annotations

import json
import os
import sys


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        data = {}

    if data.get("stop_hook_active"):
        return 0

    project_dir = os.environ.get("CLAUDE_PROJECT_DIR", ".")
    report_path = os.path.join(project_dir, "matrix_report.json")
    if not os.path.exists(report_path):
        return 0  # no matrix run recorded yet — nothing to enforce

    try:
        with open(report_path) as f:
            rows = json.load(f)
    except (json.JSONDecodeError, OSError) as e:
        print(f"verify-tests hook skipped: {e}", file=sys.stderr)
        return 0

    failures = [r for r in rows if r.get("status") != "pass"]
    if failures:
        sample = "\n".join(
            f"- {r.get('fn')} / {r.get('label')}: {r.get('detail', '')}"
            for r in failures[:10]
        )
        more = f"\n...and {len(failures) - 10} more" if len(failures) > 10 else ""
        print(json.dumps({
            "decision": "block",
            "reason": (
                f"{len(failures)} matrix case(s) still failing:\n{sample}{more}\n\n"
                "Fix them, or if a failure is because the test's expectation "
                "was wrong (not the port), update PLAN.md's 'Known deviations' "
                "and the test — then re-run the matrix before finishing. If "
                "you're at MAX_ITERATIONS, write BLOCKED.md instead of "
                "continuing to retry."
            ),
        }))
    return 0


if __name__ == "__main__":
    sys.exit(main())
