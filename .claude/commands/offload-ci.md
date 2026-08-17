---
description: Push current work to GitHub Actions and pull back results, without running the full verifier loop
---

Invoke the `ci-dispatcher` subagent to commit, push, trigger
`rust-build-test.yml` on the current branch, wait for it, and pull back the
matrix report / artifacts. Report the summary when done.
