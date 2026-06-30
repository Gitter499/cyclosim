#!/usr/bin/env bash
# Point this repo at .githooks so commit-msg strips Cursor co-author trailers.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
chmod +x .githooks/commit-msg .githooks/prepare-commit-msg
git config core.hooksPath .githooks
echo "core.hooksPath=.githooks"
