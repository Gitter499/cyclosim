#!/usr/bin/env bash
# Create a commit via commit-tree to avoid external Co-authored-by injection.
# Usage: git add … && ./scripts/git-commit-clean.sh "commit message"
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [ "$#" -lt 1 ]; then
  echo "usage: $0 \"commit message\"" >&2
  exit 1
fi

if git diff --cached --quiet; then
  echo "nothing staged" >&2
  exit 1
fi

msg="$1"
tree="$(git write-tree)"
if git rev-parse --verify HEAD >/dev/null 2>&1; then
  parent="$(git rev-parse HEAD)"
  new="$(printf '%s\n' "$msg" | git commit-tree "$tree" -p "$parent")"
else
  new="$(printf '%s\n' "$msg" | git commit-tree "$tree")"
fi
git reset --hard "$new"
echo "$new"
