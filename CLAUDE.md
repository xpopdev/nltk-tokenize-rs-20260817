# Python → Rust Port Kit

This repo drives an autonomous pipeline that takes an existing Python library,
ports its logic to Rust, exposes the Rust code as a drop-in Python package
(via PyO3 + maturin), and proves equivalence with the original through
matrix-based differential testing.

## Zero-input environment setup

Run `/setup` before anything else (or let `/port-to-rust` trigger it — see
below). It detects what's missing on this machine (Rust, maturin, gh CLI,
apt build dependencies, GitHub auth, a configured remote) and installs
everything it can itself — apt packages, Rust via rustup, the GitHub CLI,
Python tooling. It only stops to ask about things that are genuinely
impossible to script: a sudo password, the interactive `gh auth login`
flow, or which GitHub repo to push to if none is configured. Everything
else, it just does. See `.claude/commands/setup.md` and
`scripts/setup_env.sh`.

If `/port-to-rust` is run in an environment `/setup` hasn't touched yet,
check for the obvious things first (`cargo`, `maturin`, `gh auth status`)
and run `/setup` automatically rather than failing partway through a
milestone because a tool was missing.

## Pipeline (in order — do not skip stages)

1. **Explore** → `analyzer` subagent inventories the source library file by
   file and ranks public symbols by actual usage (`scripts/rank_usage.py`)
   — this ranking is what makes stage 2's milestone order "most-used
   first" instead of arbitrary.
2. **Plan** → `planner` subagent turns the inventory into a written port plan
   (`PLAN.md`), ordering milestones by usage ranking. **Stop and show the
   plan to the human before stage 3.** This is the one required approval
   gate — everything before and after it can run unattended.
3. **Port** → `rust-porter` subagent implements the Rust crate + PyO3 bindings,
   module by module, following `PLAN.md`.
4. **Test** → `matrix-test-writer` subagent generates the differential test
   suite (see "Matrix testing" below) and unit tests for anything the matrix
   can't cover (error paths, panics/exceptions, resource limits).
5. **Verify & iterate** → `verifier` subagent builds, runs the full test
   matrix, and loops: read failures → patch Rust or bindings → re-run, until
   green or until it hits `MAX_ITERATIONS` (default 8), at which point it
   reports back instead of spinning forever.
6. **Offload when heavy** → `ci-dispatcher` subagent decides whether a step
   should run locally or be pushed to GitHub Actions (see "Device awareness").

Drive the whole thing with `/setup <path-to-python-package>` on a fresh
machine (installs everything, then flows straight into the pipeline), or
`/port-to-rust <path-to-python-package>` if the environment's already set up.

## Matrix testing — what "equivalence" means here

For every public function/method in the original library:

- Generate a matrix of inputs: boundary values, empty/None/zero cases,
  large values, unicode/binary edge cases, and randomized cases (seeded, so
  runs are reproducible).
- Call the **original Python implementation** and the **new Rust-backed
  package** with identical inputs.
- Compare: return value, raised exception type + message, and (where it
  matters for the library) float tolerance / ordering / mutation of input
  arguments.
- Every mismatch is a failing test — the `verifier` agent treats it as a bug
  in the port, not in the test, unless it can show the original had a bug.
- Results get written as a matrix report (`matrix_report.md` / `.json`): one
  row per function, one column per input case, pass/fail/skip.

`scripts/gen_matrix_inputs.py` and `scripts/compare_outputs.py` are the
starting point the `matrix-test-writer` agent extends per-library.

## Device awareness (Termux / low-RAM devices)

Before any build or full test-matrix run, check the environment cheaply
(`nproc`, `free -m` or `/proc/meminfo`, and whether `$PREFIX` contains
`com.termux`). If memory is low (rule of thumb: under ~4 GB total) or the
job is one of:

- `cargo build --release` with LTO / cross-compilation for multiple targets
- `maturin build` for multiple Python versions/platforms
- the full (not smoke-subset) input matrix across all functions

...then don't run it locally. Instead:

1. Commit the current state to a working branch.
2. `git push` the branch.
3. `gh workflow run rust-build-test.yml --ref <branch>` (see
   `.github/workflows/rust-build-test.yml`).
4. `gh run watch --exit-status` to block until it finishes.
5. `gh run download` to pull back the matrix report / wheel / logs.
6. Read the results as if they'd run locally and continue the loop.

Cheap checks (`cargo check`, `clippy`, a small smoke subset of the matrix,
`cargo test` on unit tests without the full input matrix) are fine to run
locally even on a phone — keep those fast so the local loop stays usable.

## Resuming an interrupted run

Before starting or resuming `/port-to-rust`, read `STATUS.json`. It tracks
the library being ported, the working branch, whether `PLAN.md` is
approved, and each milestone's status
(`not_started` / `in_progress` / `blocked` / `done`). The `planner` agent
initializes it once `PLAN.md` exists; `verifier` updates a milestone's
status as it moves through it; `ci-dispatcher` records the last offloaded
run. If a session gets killed mid-run (phone dies, connection drops), the
next session reads `STATUS.json` and continues from the first
`not_started`/`in_progress`/`blocked` milestone instead of restarting from
milestone 1 or silently re-doing finished work.

## Enforcement beyond instructions (hooks)

Agent instructions can be forgotten under context pressure; hooks can't.
`.claude/hooks/` backs the "iterate until satisfied" and "don't ship a
broken port" rules with deterministic checks, wired via `.claude/settings.json`
and the `rust-porter`/`verifier` agents' own frontmatter:

- `rust-porter` cannot end a turn while `cargo check` is failing
  (`verify_build.py`, a `Stop` hook scoped to that agent).
- `verifier` cannot end a turn while `matrix_report.json` shows failures
  (`verify_tests.py`, scoped to that agent).
- Any `git commit` is blocked while `matrix_report.json` shows failures,
  regardless of which agent is running (`guard_commit.py`, session-wide).
- Rust/Python files are auto-formatted after every write (`format_on_write.sh`).
- A notification fires when Claude is waiting on you — e.g. the `PLAN.md`
  approval gate — via `termux-toast`/`notify-send`/`osascript` depending on
  platform (`notify.sh`).

These Stop hooks only block **once per stop attempt** (they check
`stop_hook_active` to avoid looping forever on a genuinely broken build) —
they're a deterministic nudge, not a guarantee. The `verifier` agent's own
`MAX_ITERATIONS`/`BLOCKED.md` logic is still what prevents indefinite
spinning.

## Conventions

- Branch per port run: `port/<library-name>-<date>`.
- Commits are small and reference the pipeline stage, e.g.
  `port(rust): implement parser module (stage 3/6)`.
- Every Rust module gets a matching test module before it's considered done.
- Don't delete or "simplify away" a Python edge case just because it's
  annoying to port — flag it in `PLAN.md` under "Known deviations" instead
  of silently dropping it.
- If the plan turns out to be wrong mid-port, stop, update `PLAN.md`, and
  say so — don't quietly improvise a different design.

## Subagents

| Agent | Stage | Tools |
|---|---|---|
| `analyzer` | 1. Explore | read-only |
| `planner` | 2. Plan | read-only |
| `rust-porter` | 3. Port | read/write/bash |
| `matrix-test-writer` | 4. Test | read/write/bash |
| `verifier` | 5. Verify & iterate | read/write/bash |
| `ci-dispatcher` | 6. Offload | read/bash (git, gh) |

See `.claude/agents/*.md` for each one's full brief.
