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

**M0–M2c** (trainer, HUD, Strava, ride history UI) · **M5** (Liquid Glass, workout builder, `.zwo`, highlight clips) · **M6** (Apple Music, steering) · **M7** (Zwift parity UI — in progress)

## UI layout (`Sources/VeloSim/UI/`)

| Folder | Contents |
|--------|----------|
| `Screens/` | App shell, dashboard, activities, ride flow |
| `HUD/` | In-ride overlay, controls, FTP test engine |
| `Settings/` | Settings tab + chrome helpers |
| `Components/` | Liquid Glass + HUD surface helpers |
| `Design/` | Tokens (`Tok`, `Typo`, `PowerZone`) |

Architecture: [VeloSim-Technical-Plan.md](../VeloSim-Technical-Plan.md) · UI patterns: [.cursor/skills/liquid-glass/SKILL.md](../.cursor/skills/liquid-glass/SKILL.md)
