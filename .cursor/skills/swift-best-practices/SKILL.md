---
name: swift-best-practices
description: >-
  VeloSim macOS shell Swift standards: MainActor, UniFFI bindings, Liquid Glass gating,
  Package.swift layout, non-blocking SwiftUI, and shell tests. Use when editing
  shell-macos/, SwiftUI chrome, or running shell quality passes.
---

# VeloSim Swift Best Practices

Checklist for agents working in `shell-macos/`. Community standards summarized below; full citations in [reference.md](reference.md). Liquid Glass UI details: [liquid-glass](../liquid-glass/SKILL.md).

## Community baseline

VeloSim adopts (does not duplicate) these established guides:

| Area | Standard |
|------|----------|
| API naming | [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/) |
| Formatting | [SwiftFormat](https://github.com/nicklockwood/SwiftFormat) defaults (4-space indent, consistent spacing) |
| Style conventions | [Kodeco Swift Style Guide](https://github.com/kodecocodes/swift-style-guide) + [GitHub Swift Style Guide](https://github.com/github/swift-style-guide) |

## Architecture

- **All Apple code** lives in `shell-macos/` — never in Rust crates.
- Rust interaction goes through generated UniFFI bindings (`VeloFFI`, `VeloFFIBridge`) + `VeloSimModel`.
- Platform callbacks (`SensorSourceCallback`, `TrainerControlCallback`, `MediaCaptureCallback`, …) are implemented in Swift targets.

## Naming & API design

- **Clarity at the point of use** — names read as prose at call sites (Apple API Guidelines).
- Methods: `lowerCamelCase`; types/protocols: `UpperCamelCase`.
- Omit needless words; include argument labels that clarify roles (`func move(from start: …, to end: …)`).
- Prefer `guard` + early return over deep nesting; avoid force-unwrap (`!`) in production paths.
- Access control: **`private` by default**; widen only when required by protocols or cross-file use.

## Concurrency & SwiftUI

- Mark model-bound views and coordinators `@MainActor`.
- No blocking I/O, BLE waits, or heavy encode in `body` — use `Task`, async helpers, or dedicated services.
- Prefer structured concurrency (`async`/`await`, `TaskGroup`) over raw GCD in new code.
- Keep `ContentView` thin; extract chrome to `Sources/VeloSim/UI/`.
- Use `MARK: -` sections to group extensions and protocol conformances.

## Liquid Glass

- Gate with `#if VELO_LIQUID_GLASS` (defined in `Package.swift` when macOS 26 SDK is available).
- Use shared helpers in `UI/Components/VeloGlass.swift` and `UI/Components/HUDSurface.swift` — no copy-pasted availability checks.
- Follow [docs/VeloSim-UI-and-Zwift-Parity-Guide.md](../../../docs/VeloSim-UI-and-Zwift-Parity-Guide.md) — never `.ultraThinMaterial` as fake glass.

## Package.swift

- `VeloSimSupport` compiles shared app logic; executable target excludes UI files duplicated elsewhere (see `supportExclude`).
- `Ride/` exclude on executable is intentional — compiled via `VeloSimSupport`.
- Do not link Apple frameworks into Rust; link only from Swift targets.

## UniFFI usage

- Regenerate after FFI changes: `just bindgen`.
- DTOs (`WorkoutDto`, `RideSummaryDto`, …) come from generated stubs — don't hand-roll parallel types.
- Highlight encode: implement `MediaCaptureCallback.encode_highlight_clip` in shell (VideoToolbox).

## Documentation

- Public types and non-obvious methods: `///` summary using Swift markup.
- File-level `// MARK:` headers for large files (builder, ride flow).

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
- [ ] API names clear at call site (Apple Guidelines)
- [ ] private by default; no force-unwrap in prod paths
- [ ] @MainActor on UI-bound types; no blocking in body
- [ ] UniFFI types used; bindgen run if FFI changed
- [ ] Liquid Glass via VeloGlass helpers + VELO_LIQUID_GLASS gate
- [ ] New UI in UI/ or Support target; Package.swift excludes correct
- [ ] VeloSimTests added for non-trivial shell logic
- [ ] lint-shell-ui.sh green
```

## Additional resources

- Adopted rules + source links: [reference.md](reference.md)
- Liquid Glass patterns: [liquid-glass](../liquid-glass/SKILL.md)
- Shell layout: [shell-macos/README.md](../../../shell-macos/README.md)
- Contributor workflow: [AGENTS.md](../../../AGENTS.md)
