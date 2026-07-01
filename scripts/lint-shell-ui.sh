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
if [[ ! -f "$SOURCES/UI/Components/VeloGlass.swift" ]]; then
  echo "error: missing $SOURCES/UI/Components/VeloGlass.swift (add shared glass helpers per docs/VeloSim-UI-and-Zwift-Parity-Guide.md)" >&2
  exit 1
fi

if [[ ! -f "$SOURCES/UI/Components/HUDSurface.swift" ]]; then
  echo "error: missing $SOURCES/UI/Components/HUDSurface.swift (HUD legibility helper per guide §8)" >&2
  exit 1
fi

echo "lint-shell-ui: anti-pattern scan…"
FAIL=0

# Fake glass — never use material as a glass stand-in in HUD/Components paths (guide §10)
HUD_UI_PATHS=("$SOURCES/UI/HUD" "$SOURCES/UI/Components")
for dir in "${HUD_UI_PATHS[@]}"; do
  if [[ -d "$dir" ]] && rg -n '\.ultraThinMaterial' "$dir" --glob '*.swift' 2>/dev/null | rg -v '^\s*.*//' >/dev/null 2>&1; then
    echo "error: .ultraThinMaterial in $dir — use hudSurface / .glassEffect only (guide §10)" >&2
    rg -n '\.ultraThinMaterial' "$dir" --glob '*.swift' 2>/dev/null | rg -v '^\s*.*//' >&2 || true
    FAIL=1
  fi
done

# Glass on full list rows (common mistake) — flag bare .glassEffect on ForEach rows
if rg -n '\.glassEffect\(' "$SOURCES" --glob '*.swift' 2>/dev/null | rg -i 'ForEach|LazyVStack|List' >/dev/null 2>&1; then
  echo "warning: .glassEffect may be applied inside list content — prefer chrome layer only" >&2
fi

# Require @available when using macOS 26-only APIs outside Components/VeloGlass.swift and HUDSurface.swift
while IFS= read -r hit; do
  file="${hit%%:*}"
  if [[ "$file" == *"/UI/Components/VeloGlass.swift" ]] || [[ "$file" == *"/UI/Components/HUDSurface.swift" ]]; then
    continue
  fi
  if ! rg -q '@available\(macOS 26|#available\(macOS 26' "$file" 2>/dev/null; then
    echo "error: $file uses glass APIs without macOS 26 availability guard (centralize in Components/ or add #available)" >&2
    FAIL=1
  fi
done < <(rg -l 'GlassEffectContainer|\.glassEffect\(|glassEffectID' "$SOURCES" --glob '*.swift' 2>/dev/null || true)

if [[ "$FAIL" -ne 0 ]]; then
  exit 1
fi

echo "lint-shell-ui: OK"
