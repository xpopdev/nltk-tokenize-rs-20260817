# rust-port-kit

A Claude Code project scaffold that takes an existing Python library,
autonomously plans and executes a port of it to Rust — most-used code
first — packages the Rust code as a drop-in Python package (PyO3 +
maturin), and proves it behaves identically to the original via
matrix-based differential testing. One command (`/setup`) installs
everything the pipeline needs on a bare Ubuntu box (Termux also
supported); anything too heavy to build/test locally gets offloaded to
GitHub Actions.

## What's in here

```
CLAUDE.md                       pipeline description Claude Code reads automatically
STATUS.json                     resumability: which milestone a run is on, if interrupted
Makefile                        short local commands: make check / test-smoke / offload ...
.claude/settings.json           permission allowlist + hooks wiring
.claude/hooks/                  deterministic enforcement (see "Hooks" below)
.claude/agents/                 6 subagents, one per pipeline stage
.claude/commands/                /setup, /port-to-rust, /analyze-lib, /run-matrix-tests, /offload-ci
.github/workflows/
  rust-build-test.yml           GH Actions: build + full matrix, triggered via gh CLI
  release-wheels.yml            builds installable wheels for Linux/macOS/Windows on a tag
scripts/
  setup_env.sh                  installs everything (apt/rustup/pip), Ubuntu-first, Termux-aware
  rank_usage.py                 ranks public symbols by real usage -> milestone order
  gen_matrix_inputs.py          input-matrix generator skeleton
  compare_outputs.py            differential test runner + correctness report writer
  benchmark.py                  original-vs-ported timing report
  dispatch_and_wait.sh          manual "push, trigger CI, wait, pull results" helper
Cargo.toml / pyproject.toml / src/lib.rs
                                 starter PyO3 + maturin crate, with a proptest example
                                 (replace src/lib.rs's example with real ported code)
```

## Setup

On a fresh machine, this is the whole setup step:

```
/setup path/to/your_python_package
```

It installs everything it can by itself — Rust via rustup, build
dependencies via apt (build-essential, pkg-config, libssl-dev, etc.), the
GitHub CLI, maturin/pytest/hypothesis — and only stops to ask about things
that are genuinely impossible to script: an interactive `gh auth login`,
a sudo password it can't type, or which GitHub repo to use if none is
configured yet. Once that's resolved, it flows straight into the port
pipeline for the library you gave it — no separate second command needed.
Run `/setup` with no argument if you just want the environment ready
without starting a port yet; run `make setup` for the same thing from a
plain shell.

If you'd rather do it by hand: install Rust + maturin + gh, run
`gh auth login`, and push this repo to GitHub so `.github/workflows/*.yml`
are on the default branch (required for `gh workflow run` to be able to
dispatch them).

## Running a port

```
/port-to-rust path/to/your_python_package
```

This runs analysis → plan, then **stops and shows you `PLAN.md` for
approval** before writing any Rust. The analysis step ranks every public
function by how often it's actually called (`scripts/rank_usage.py`), and
the plan orders milestones by that ranking — the most-used code gets
ported (and gets the most test scrutiny) first, instead of an arbitrary
file-by-file order. Reply to continue and it works through each milestone
(port → generate matrix tests → build/test/iterate → offload heavy steps
to CI as needed) until the whole library is green.

You can also run stages individually:
- `/analyze-lib <path>` — just the inventory + usage ranking.
- `/run-matrix-tests smoke` or `/run-matrix-tests full` — just the test run.
- `/offload-ci` — manually push + trigger the GitHub Actions build.

If a run gets interrupted (closed terminal, dead connection), the next
`/port-to-rust` reads `STATUS.json` and resumes from the first
unfinished milestone instead of starting over.

## Hooks: enforcement, not just instructions

Agent instructions can get lost under context pressure; hooks can't.
`.claude/hooks/` backs a few rules with actual checks, wired via
`.claude/settings.json` and two agents' own frontmatter:

- `rust-porter` can't end its turn while `cargo check` is failing.
- `verifier` can't end its turn while `matrix_report.json` shows failures.
- Any `git commit` is blocked while `matrix_report.json` shows failures,
  regardless of which agent runs it.
- Rust/Python files auto-format on every write.
- You get a notification (`termux-toast`/`notify-send`/`osascript`,
  whichever exists) when Claude's waiting on you — e.g. the `PLAN.md`
  approval gate.

## Benchmarking

`python scripts/benchmark.py` (or `make bench`) times the original against
the ported implementation for every function registered in
`compare_outputs.py`'s `FUNCTION_PAIRS`, and writes `benchmark_report.md`.
Correctness (the matrix) and speed are tracked separately on purpose — a
port that's correct but not actually faster is worth flagging before
calling a milestone "done".

## Running on a phone (Termux)

Nothing here requires you to change how you work — the `verifier` agent
checks available RAM before deciding whether to build/test locally or hand
off. Heavy steps get pushed to `port/*` branches and run on GitHub's
runners instead of your device; `gh run watch` blocks until they finish and
`gh run download` pulls the report/wheel back down. If you'd rather trigger
that by hand instead of letting the agent decide, `scripts/dispatch_and_wait.sh
[smoke|full]` does the same thing as a plain shell script.

## Recommended Claude Code plugins

Install from the official marketplace (`/plugin marketplace add
claude-plugins-official` if it isn't already registered, then `/plugin
install <name>@claude-plugins-official`):

| Plugin | Why |
|---|---|
| `rust-analyzer-lsp` | Real-time Rust diagnostics, jump-to-def, catches type errors as `rust-porter` writes code, before a build is even needed. Note: can be memory-hungry — disable it (`/plugin disable rust-analyzer-lsp`) if it's straining a 4GB device. |
| `pyright-lsp` | Same, for the Python side — useful while `analyzer`/`matrix-test-writer` are reading/writing Python. |
| `github` | Native GitHub integration (issues/PRs) alongside the `gh` CLI calls the agents already make. |
| `pr-review-toolkit` | Structured review of the PR each port run produces, before you merge. |
| `commit-commands` | Consistent commit message formatting matching the `CLAUDE.md` convention. |

Verify exact names/availability with `/plugin marketplace list` since the
catalog changes — the table above is a starting point, not a guarantee of
what's current.

## A note on autonomy

`.claude/settings.json` allowlists the whole toolchain the pipeline needs —
git, gh (including `gh auth login` and `gh repo create`), cargo, maturin,
python, pytest, and, for `/setup`, `apt-get`/`sudo -v`/`curl`/`dpkg` — so
none of it prompts you step by step, plus a sandbox and a short deny list
for genuinely dangerous commands (force-push, `rm -rf /`, piping `curl`
straight into a shell — `setup_env.sh` installs Rust by downloading
`rustup-init` and running it as a separate step specifically to avoid
needing an exception to that rule). That's deliberately short of full
`--dangerously-skip-permissions` bypass mode — if you want that anyway
(e.g. running unattended in a container you don't mind rebuilding), it's a
one-line change, but it also removes the safety net for things like an
errant `rm -rf` this list doesn't anticipate. Your call.
