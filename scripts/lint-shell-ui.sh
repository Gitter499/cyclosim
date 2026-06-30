#!/usr/bin/env bash
# Static + compile checks for shell-macos UI quality (Liquid Glass conventions).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SHELL_DIR="$ROOT/shell-macos"
SOURCES="$SHELL_DIR/Sources/VeloSim"

cd "$SHELL_DIR"

echo "lint-shell-ui: swift build (VeloSim + VeloSimSupport)…"
swift build --product VeloSim 2>&1
swift build --product VeloSimSupport 2>&1

echo "lint-shell-ui: checking Liquid Glass helper exists…"
if [[ ! -f "$SOURCES/UI/VeloGlass.swift" ]]; then
  echo "error: missing $SOURCES/UI/VeloGlass.swift (add shared glass helpers per .cursor/skills/liquid-glass/SKILL.md)" >&2
  exit 1
fi

echo "lint-shell-ui: anti-pattern scan…"
FAIL=0

# Glass on full list rows (common mistake) — flag bare .glassEffect on ForEach rows
if rg -n '\.glassEffect\(' "$SOURCES" --glob '*.swift' 2>/dev/null | rg -i 'ForEach|LazyVStack|List' >/dev/null 2>&1; then
  echo "warning: .glassEffect may be applied inside list content — prefer chrome layer only" >&2
fi

# Require @available when using macOS 26-only APIs outside VeloGlass.swift
while IFS= read -r hit; do
  file="${hit%%:*}"
  if [[ "$file" == *"/UI/VeloGlass.swift" ]]; then
    continue
  fi
  if ! rg -q '@available\(macOS 26|#available\(macOS 26' "$file" 2>/dev/null; then
    echo "error: $file uses glass APIs without macOS 26 availability guard (centralize in VeloGlass.swift or add #available)" >&2
    FAIL=1
  fi
done < <(rg -l 'GlassEffectContainer|\.glassEffect\(|glassEffectID' "$SOURCES" --glob '*.swift' 2>/dev/null || true)

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "lint-shell-ui: OK"
