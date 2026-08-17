---
description: Run the differential matrix test suite at a given size and report pass/fail
argument-hint: [smoke|full]
---

Mode: $ARGUMENTS (default to `smoke` if empty).

Check device resources first (see `CLAUDE.md` → "Device awareness"). If mode
is `full` and this device looks too constrained, don't run it locally —
invoke `ci-dispatcher` instead and explain why. Otherwise run
`scripts/compare_outputs.py` at the requested size directly, then print the
`matrix_report.md` summary (totals, and the top few failing rows if any).
