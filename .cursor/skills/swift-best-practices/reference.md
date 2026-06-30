# VeloSim Swift — adopted community rules

Rules below are distilled from high-star community standards (star counts from 2026-06-30). VeloSim-specific layout patterns follow.

## Source repositories

| Source | Stars | URL |
|--------|------:|-----|
| swiftlang/swift | 70,124 | https://github.com/swiftlang/swift |
| swiftlang/swift-evolution (SE-0023) | 15,854 | https://github.com/swiftlang/swift-evolution/blob/main/proposals/0023-api-guidelines.md |
| kodecocodes/swift-style-guide | 13,167 | https://github.com/kodecocodes/swift-style-guide |
| nicklockwood/SwiftFormat | 8,840 | https://github.com/nicklockwood/SwiftFormat |
| github/swift-style-guide (archived) | 4,760 | https://github.com/github/swift-style-guide |
| airbnb/swift | 2,725 | https://github.com/airbnb/swift |
| swiftlang/swift-testing | 2,155 | https://github.com/swiftlang/swift-testing |
| swiftlang/swift-org-website (API Guidelines source) | 553 | https://github.com/swiftlang/swift-org-website/blob/main/documentation/api-design-guidelines/index.md |
| Apple API Design Guidelines (official) | — | https://swift.org/documentation/api-design-guidelines/ |

## API design (Apple + SE-0023)

- **Clarity at the point of use** is the primary goal.
- Omit needless words; role-based names (`addObserver`, not `add`).
- Argument labels clarify parameter roles; omit `_` only when no ambiguity.
- Methods that return `Void` read as side-effect verbs; value-returning methods read as nouns.
- Prefer methods/properties over free functions for domain operations.
- Use Swift's `///` markup for public API documentation.

## Naming (Kodeco + GitHub style guides)

- Types/protocols/enums: `UpperCamelCase`.
- Functions/variables/properties: `lowerCamelCase`.
- Constants: `lowerCamelCase` (Swift convention) unless mirroring C/ObjC APIs.
- Acronyms in names: capitalize all letters in short acronyms (`URL`, `UUID`); capitalize first only in longer ones (`UrlSession` → prefer `Url` per Apple).
- Boolean properties read as assertions: `isEnabled`, `hasPrefix`, `canRetry`.

## Formatting (SwiftFormat + Kodeco)

- 4-space indentation; no tabs.
- Opening brace on same line as declaration.
- One type per file (extensions may follow in same file with `MARK:`).
- Max line length ~120 (SwiftFormat default); break long chains and argument lists.
- Single space after commas; no trailing whitespace.

## Access control (GitHub + Airbnb guides)

- Default to `private`; use `fileprivate` only when extensions in same file need access.
- `internal` is implicit — omit keyword unless clarifying intent.
- `public`/`open` only for library surfaces (not typical in VeloSim shell).
- Mark classes `final` when subclassing isn't intended.

## Safety & control flow (GitHub + Kodeco)

- Avoid force-unwrap (`!`) and force-cast (`as!`) in production code — use `guard let` / `if let`.
- Prefer `guard` for early exits; keep happy path unindented.
- Prefer `let` over `var`; use `var` only when mutation is required.
- Use trailing closure syntax when the closure is the final argument.

## Organization (Airbnb + Kodeco)

- `// MARK: - Section` headers for large files.
- Extensions group protocol conformances separately from primary type body.
- Keep types focused; extract subviews and helpers rather than 500-line views.

## Concurrency (Apple + modern Swift)

- UI-bound code: `@MainActor`.
- Prefer `async`/`await` over callback/GCD for new async work.
- Avoid `DispatchQueue.main.sync` — never block the main thread.
- Capture `[weak self]` in long-lived closures when retaining `self` would create cycles.

## Testing (swift-testing + existing XCTest)

- Shell tests use XCTest in `shell-macos/Tests/VeloSimTests/`.
- Test names describe behavior: `testParseFTMSPowerMeasurement_validPayload`.
- One assertion concept per test when practical; use fixtures for BLE payloads.

## VeloSim-specific patterns

### Target layout

| Target | Role |
|--------|------|
| `VeloSim` | App executable (SwiftUI entry) |
| `VeloSimSupport` | Model, ride flow, Strava, screenshot/clip encode |
| `VeloSimBLE` | CoreBluetooth FTMS |
| `VeloFFIBridge` | C bridge to `libvelo_ffi` |

### VELO_LIQUID_GLASS

`Package.swift` probes SDK version at package-load time:

```swift
private let liquidGlassSwiftSettings: [SwiftSetting] =
    macOSSDKSupportsLiquidGlass() ? [.define("VELO_LIQUID_GLASS")] : []
```

CI runners (macOS 14) compile without Liquid Glass APIs; local Xcode 26+ gets real `glassEffect`.

### VeloGlass helper pattern

```swift
#if VELO_LIQUID_GLASS
self.glassEffect(.regular, in: shape)
#else
self.background(.ultraThinMaterial, in: shape)
#endif
```

### Anti-patterns (lint-shell-ui.sh scans)

- `DispatchQueue.main.sync` in view code
- Duplicate Liquid Glass availability checks outside `VeloGlass.swift`
- Missing `VeloGlass.swift` helper file

### Workout builder

- Builder UI in `WorkoutBuilderView.swift`; ERG targets driven by Rust `WorkoutEngine` via FFI.
- `.zwo` import: `parse_zwo_xml` FFI → `WorkoutDto` → start ride.
