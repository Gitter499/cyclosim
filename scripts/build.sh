#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="$ROOT/target"

cd "$ROOT"
cargo build --release -p velo-ffi
cargo run -p velo-ffi --bin uniffi-bindgen -- generate \
  --library "$CARGO_TARGET_DIR/release/libvelo_ffi.dylib" \
  --language swift \
  --out-dir "$ROOT/shell-macos/Generated"

cp "$ROOT/shell-macos/Generated/velo_ffiFFI.h" "$ROOT/shell-macos/Bridge/include/velo_ffiFFI.h"

cd "$ROOT/shell-macos"
swift build -c release \
  -Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker "$ROOT/shell-macos/Info.plist"

echo "Built: $ROOT/shell-macos/.build/release/VeloSim"
