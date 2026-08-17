---
description: Run the full Python→Rust port pipeline against a target library path
argument-hint: <path-to-python-package>
---

Target library: $ARGUMENTS

Before anything else, cheaply confirm the environment is ready: `cargo
--version`, `python3 -c "import maturin"`, and `gh auth status`. If any of
these fail, run `/setup` first (no need to ask permission — this is exactly
what `/setup` exists for) rather than pushing ahead and failing partway
through a later milestone. Once the environment's confirmed ready, continue
below.

Run the pipeline described in `CLAUDE.md`, in order:

1. Invoke the `analyzer` subagent on the target library to produce
   `ANALYSIS.md`.
2. Invoke the `planner` subagent to produce `PLAN.md` from `ANALYSIS.md`.
3. Show me a short summary of `PLAN.md` and **stop and wait for my
   approval** before doing anything else. Do not write any Rust or start
   stage 3 in this same run.

Once I approve in a follow-up message, continue with:

4. Invoke `rust-porter` for the first milestone in `PLAN.md`.
5. Invoke `matrix-test-writer` for that milestone.
6. Invoke `verifier` to build/test/iterate that milestone to green,
   offloading to `ci-dispatcher` for anything heavy per `CLAUDE.md`'s
   device-awareness rule.
7. Repeat 4–6 for each remaining milestone in `PLAN.md`, in order.
8. When every milestone is green, produce a final summary: what was
   ported, the final `matrix_report.md` totals, anything listed under
   "Known deviations", and the branch/PR to review.

If any stage's agent reports it's blocked (see the `verifier` agent's stop
conditions), stop the pipeline and report the blocker instead of skipping
ahead to later milestones.
