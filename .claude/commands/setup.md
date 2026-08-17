---
description: Detect and install everything the pipeline needs (Ubuntu-first, Termux-aware). Only asks about things that genuinely can't be automated.
argument-hint: [path-to-python-package]
---

Optional target library for after setup: $ARGUMENTS

1. Run `bash scripts/setup_env.sh` and read the `SETUP_SUMMARY:` JSON line
   it prints at the end. It already attempted to install everything it
   could (apt packages, Rust via rustup, gh CLI, maturin/pytest/hypothesis)
   — don't re-attempt anything it already tried.

2. If `needs_user` is empty, skip straight to step 4.

3. If `needs_user` is non-empty, handle each item:
   - **`gh auth login`** — tell the person to run it themselves (it's an
     interactive/browser or device-code flow). **Never ask them to paste a
     token or password into chat** — if they do anyway, don't echo it back
     or write it to a file; just proceed once `gh auth status` succeeds.
   - **sudo password needed** — tell them to run `sudo -v` themselves (or
     grant passwordless sudo, or install the listed packages manually),
     then say to re-run `/setup`.
   - **Network/install failures** (rustup, gh apt source, pip) — show the
     specific error from the log file the script pointed to, and ask
     whether they want to troubleshoot it or install that piece manually.
   - **No GitHub remote configured** — check `git remote get-url origin`;
     if it fails, ask whether to `gh repo create` a new repo (get a
     name/visibility from them) or `git remote add origin <url>` an
     existing one they'll provide.

   Ask about all outstanding items in one message rather than one at a
   time. Once the person responds, re-run `bash scripts/setup_env.sh` to
   confirm — don't just take their word for it — and only move on once
   `SETUP_SUMMARY` reports `"ready": true` (git remote, being outside this
   script's scope, is confirmed separately via `git remote get-url origin`).

4. Make sure the workflow files are actually usable: `git remote get-url
   origin` must resolve, and `.github/workflows/*.yml` must be pushed to
   the repo's default branch (GitHub won't let `gh workflow run` dispatch a
   workflow that only exists on a feature branch). If they're not pushed
   yet, commit and push them to the default branch now.

5. Report a short summary: what's installed, what was already present,
   and confirm the environment is ready for `/port-to-rust`.

6. If `$ARGUMENTS` was given, continue immediately into the same flow as
   `/port-to-rust $ARGUMENTS` (analyzer → planner → **stop for plan
   approval**) — the person shouldn't need to type a second command. If
   `$ARGUMENTS` was empty, just stop after the step-5 summary.
