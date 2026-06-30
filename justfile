set shell := ["bash", "-cu"]

export CARGO_TARGET_DIR := justfile_directory() + "/target"

default:
    @just build

build:
    ./scripts/build.sh

test:
    cargo test
    cargo build --release -p velo-ffi
    cd shell-macos && swift test

lint:
    ./scripts/lint-apple-symbols.sh
    ./scripts/lint-shell-ui.sh

bindgen:
    cargo build --release -p velo-ffi
    cargo run -p velo-ffi --bin uniffi-bindgen -- generate \
      --library target/release/libvelo_ffi.dylib \
      --language swift \
      --out-dir shell-macos/Generated

run:
    ./scripts/build.sh
    ./shell-macos/.build/release/VeloSim
