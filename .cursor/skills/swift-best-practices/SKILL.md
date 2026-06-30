---
name: swift-best-practices
description: >-
  VeloSim macOS shell Swift standards: MainActor, UniFFI bindings, Liquid Glass gating,
  Package.swift layout, non-blocking SwiftUI, and shell tests. Use when editing
  shell-macos/, SwiftUI chrome, or running shell quality passes.
---

# VeloSim Swift Best Practices

Checklist for agents working in `shell-macos/`. Liquid Glass UI details: [liquid-glass](../liquid-glass/SKILL.md).

## Architecture

- **All Apple code** lives in `shell-macos/` — never in Rust crates.
- Rust interaction goes through generated UniFFI bindings (`VeloFFI`, `VeloFFIBridge`) + `VeloSimModel`.
- Platform callbacks (`SensorSourceCallback`, `TrainerControlCallback`, `MediaCaptureCallback`, …) are implemented in Swift targets.

## Concurrency & SwiftUI

- Mark model-bound views and coordinators `@MainActor`.
- No blocking I/O, BLE waits, or heavy encode in `body` — use `Task`, async helpers, or dedicated services.
- Keep `ContentView` thin; extract chrome to `Sources/VeloSim/UI/`.

## Liquid Glass

- Gate with `#if VELO_LIQUID_GLASS` (defined in `Package.swift` when macOS 26 SDK is available).
- Use shared helpers in `VeloGlass.swift` — no copy-pasted availability checks.
- Glass on **chrome only** (toolbars, sheets, action bars) — not Metal viewport or in-game HUD.

## Package.swift

- `VeloSimSupport` compiles shared app logic; executable target excludes UI files duplicated elsewhere (see `supportExclude`).
- `Ride/` exclude on executable is intentional — compiled via `VeloSimSupport`.
- Do not link Apple frameworks into Rust; link only from Swift targets.

## UniFFI usage

- Regenerate after FFI changes: `just bindgen`.
- DTOs (`WorkoutDto`, `RideSummaryDto`, …) come from generated stubs — don't hand-roll parallel types.
- Highlight encode: implement `MediaCaptureCallback.encode_highlight_clip` in shell (VideoToolbox).

## Tests

Location: `shell-macos/Tests/VeloSimTests/`

| File | Coverage |
|------|----------|
| `FTMSParserTests` | BLE protocol |
| `StravaOAuthTests` / `StravaUploadTests` | OAuth + upload |
| `HighlightClipEncoderTests` | Clip encode path |
| `RideFlowTests` / `RideSummaryFormattingTests` | Ride lifecycle UI helpers |

Run: `just test` or `swift test` (requires built `libvelo_ffi.dylib`).

## Before commit

```bash
./scripts/lint-shell-ui.sh
just test   # optional full Rust + Swift
```

Commit via `./scripts/git-commit-clean.sh "message"`.

## Checklist

```
Swift pass:
- [ ] @MainActor on UI-bound types; no blocking in body
- [ ] UniFFI types used; bindgen run if FFI changed
- [ ] Liquid Glass via VeloGlass helpers + VELO_LIQUID_GLASS gate
- [ ] New UI in UI/ or Support target; Package.swift excludes correct
- [ ] VeloSimTests added for non-trivial shell logic
- [ ] lint-shell-ui.sh green
```

## Additional resources

- Liquid Glass patterns: [liquid-glass](../liquid-glass/SKILL.md)
- Shell layout: [shell-macos/README.md](../../../shell-macos/README.md)
- Contributor workflow: [AGENTS.md](../../../AGENTS.md)
