# shell-macos

macOS Swift shell — window/UI, CoreBluetooth FTMS, Strava OAuth, and UniFFI bridge to `velo-ffi`.

## Targets

| Target | Role |
|--------|------|
| `VeloSim` | App executable (SwiftUI) |
| `VeloSimBLE` | FTMS protocol + trainer bridge |
| `VeloSimSupport` | Platform callbacks, Strava upload, PNG encode |
| `VeloFFI` / `VeloFFIBridge` | Generated + C bridge to Rust static lib |

All Apple-only code lives here — not in Rust crates.

## Build

```bash
just build    # Rust release lib + Swift package
just run
just test     # cargo test + swift test
```

Requires Xcode Swift toolchain and a built `target/release/libvelo_ffi.dylib`.

## Milestone

**M0–M2c** (trainer, HUD, Strava, ride history UI)

Architecture: [VeloSim-Technical-Plan.md](../VeloSim-Technical-Plan.md)
