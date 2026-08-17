#!/usr/bin/env bash
# Manual equivalent of what the ci-dispatcher subagent does: push the
# current branch, trigger the heavy build+test workflow on GitHub Actions,
# block until it's done, and pull the results back down.
#
# Usage: scripts/dispatch_and_wait.sh [smoke|full]
set -euo pipefail

MODE="${1:-smoke}"
BRANCH="$(git rev-parse --abbrev-ref HEAD)"

if [[ "$BRANCH" == "main" || "$BRANCH" == "master" ]]; then
  echo "Refusing to dispatch CI directly from $BRANCH — create a port/* branch first." >&2
  exit 1
fi

echo "Pushing $BRANCH..."
git push -u origin "$BRANCH"

echo "Triggering rust-build-test.yml (mode=$MODE)..."
gh workflow run rust-build-test.yml --ref "$BRANCH" -f mode="$MODE"

echo "Waiting for the run to start..."
sleep 5
RUN_ID="$(gh run list --workflow=rust-build-test.yml --branch "$BRANCH" --limit 1 --json databaseId --jq '.[0].databaseId')"

echo "Watching run $RUN_ID..."
gh run watch "$RUN_ID" --exit-status

echo "Downloading artifacts..."
gh run download "$RUN_ID"

echo "Done. See matrix-report/matrix_report.md"
