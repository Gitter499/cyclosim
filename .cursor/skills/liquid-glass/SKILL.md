---
name: liquid-glass
description: >-
  VeloSim macOS shell Liquid Glass UI patterns (SwiftUI, macOS 26+ with fallbacks).
  Use when building or refactoring shell chrome — setup flows, sidebars, sheets,
  ride summary, workout builder chrome — or when the user mentions Liquid Glass.
---

# Liquid Glass — VeloSim shell

Apple's Liquid Glass is for the **navigation / chrome layer** that floats above content. In VeloSim, **only `shell-macos`** uses it — never `velo-render` HUD or Rust crates.

## VeloSim boundaries

| Use Liquid Glass | Do NOT use Liquid Glass |
|------------------|-------------------------|
| Sidebar chrome, toolbars, floating controls | Metal ride viewport (`MetalRideView`) |
| Setup / pairing sheets, ride summary sheet | In-game HUD (Rust `velo-render`) |
| Workout builder **chrome** (headers, action bar) | Full-screen list/table backgrounds |
| Settings, segmented pickers in chrome | Stacking glass on every row in a list |

Content stays on solid or subtle materials (`.background(.quaternary)`). Glass wraps **controls**, not scrollable content bodies.

## API (macOS 26 / Xcode 26)

Gate with availability; fall back on macOS 14–25:

```swift
@ViewBuilder
func veloGlassChrome<S: Shape>(_ shape: S = Capsule()) -> some View {
    if #available(macOS 26, *) {
        self.glassEffect(.regular, in: shape)
    } else {
        self.background(.ultraThinMaterial, in: shape)
    }
}
```

Prefer shared helpers in `shell-macos/Sources/VeloSim/UI/VeloGlass.swift` — do not copy-paste availability checks.

### Core rules (Apple WWDC 2025)

1. **Navigation layer only** — toolbars, sidebars, sheets, FABs; not list rows or media.
2. **Never stack glass on glass** — one glass layer per visual stack.
3. **`GlassEffectContainer`** when multiple glass controls sit together (toolbar button groups, action bars).
4. **`.regular` default** — do not mix `.regular` and `.clear` in the same group.
5. **`.interactive()`** only on tappable controls (buttons, toggles), not static labels.
6. **Tint sparingly** — primary actions only (Start ride, Connect Strava).
7. **Morphing** — `@Namespace` + `glassEffectID(_:in:)` for show/hide of related chrome.
8. **Accessibility** — trust system Reduce Transparency / Reduce Motion; test both.

## VeloSim layout pattern

```
HSplitView
├── MetalRideView          ← content (no glass)
└── Sidebar chrome         ← GlassEffectContainer + sections
    ├── Setup toolbar
    ├── Section cards (solid .quaternary interior)
    └── Ride summary sheet (glass header + solid stats body)
```

- Remove redundant `GroupBox` backgrounds when glass chrome replaces them.
- Section **interiors** use rounded rects with `.quaternary` — glass on the **section header bar** or **floating action row** only.

## Components to reuse

| Helper | Purpose |
|--------|---------|
| `VeloGlass.swift` | Availability-safe modifiers |
| `SetupChromeView` | Pre-ride: sensors, route, bike, FTP, workout entry |
| `RideSummarySheet` | Post-ride stats + publish badge + open folder |

Extract new chrome into `shell-macos/Sources/VeloSim/UI/` — keep `ContentView` thin.

## SwiftUI quality checklist

Before opening a PR:

- [ ] `@MainActor` on model-bound views; no blocking work in `body`
- [ ] Bindings go through `VeloSimModel` methods, not raw `handle` in views
- [ ] `monospacedDigit()` on power/time stats
- [ ] Error strings surfaced to user (red caption), not silent failures
- [ ] Previews for new views (`#Preview`) where layout is non-trivial
- [ ] `Package.swift` exclude list updated if files land under `VeloSimSupport` path
- [ ] Run `./scripts/lint-shell-ui.sh` locally

## Verification

```bash
cd shell-macos && swift build --product VeloSim
./scripts/lint-shell-ui.sh
cd shell-macos && swift test
```

## References

- Technical plan §12 (`VeloSim-Technical-Plan.md`) — Liquid Glass owns setup/summary, not HUD
- [WWDC25 — Build a SwiftUI app with the new design](https://developer.apple.com/videos/play/wwdc2025/323/)
