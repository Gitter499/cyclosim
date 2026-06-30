#!/usr/bin/env bash
# Fail if Apple-only symbols leak below shell-macos/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PATTERN='CoreBluetooth|MusicKit|AppKit|CoreFoundation\.|Metal\.|CMHeadphone'

fail=0
while IFS= read -r crate; do
  if rg -q "$PATTERN" "$crate/src" --glob '*.rs' 2>/dev/null; then
    # Ignore comment-only hits
    if rg "$PATTERN" "$crate/src" --glob '*.rs' | rg -qv '^\s*//'; then
      echo "Apple symbol leak in $crate"
      rg "$PATTERN" "$crate/src" --glob '*.rs' | rg -v '^\s*//' || true
      fail=1
    fi
  fi
done < <(find "$ROOT/crates" -maxdepth 1 -type d \( -name 'velo-core' -o -name 'velo-units' -o -name 'velo-platform' \))

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

echo "Apple symbol lint passed."
