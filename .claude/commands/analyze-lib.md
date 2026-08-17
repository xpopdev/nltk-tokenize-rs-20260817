---
description: Re-run just the analysis stage against a target library (e.g. after the source changed)
argument-hint: <path-to-python-package>
---

Invoke the `analyzer` subagent on $ARGUMENTS and regenerate `ANALYSIS.md`.
If `ANALYSIS.md` already exists, diff the new findings against it and call
out what changed rather than silently overwriting context that `PLAN.md`
may depend on.
