# VeloSim Swift — reference patterns

## Target layout

| Target | Role |
|--------|------|
| `VeloSim` | App executable (SwiftUI entry) |
| `VeloSimSupport` | Model, ride flow, Strava, screenshot/clip encode |
| `VeloSimBLE` | CoreBluetooth FTMS |
| `VeloFFIBridge` | C bridge to `libvelo_ffi` |

## VELO_LIQUID_GLASS

`Package.swift` probes SDK version at package-load time:

```swift
private let liquidGlassSwiftSettings: [SwiftSetting] =
    macOSSDKSupportsLiquidGlass() ? [.define("VELO_LIQUID_GLASS")] : []
```

CI runners (macOS 14) compile without Liquid Glass APIs; local Xcode 26+ gets real `glassEffect`.

## VeloGlass helper pattern

```swift
#if VELO_LIQUID_GLASS
self.glassEffect(.regular, in: shape)
#else
self.background(.ultraThinMaterial, in: shape)
#endif
```

## Anti-patterns (lint-shell-ui.sh scans)

- `DispatchQueue.main.sync` in view code
- Duplicate Liquid Glass availability checks outside `VeloGlass.swift`
- Missing `VeloGlass.swift` helper file

## Workout builder

- Builder UI in `WorkoutBuilderView.swift`; ERG targets driven by Rust `WorkoutEngine` via FFI.
- `.zwo` import: `parse_zwo_xml` FFI → `WorkoutDto` → start ride.
