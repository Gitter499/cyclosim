---
name: velo-ui-parity
description: >-
  VeloSim shell UI and Zwift parity — follow docs/VeloSim-UI-and-Zwift-Parity-Guide.md.
  Use when restructuring shell chrome, HUD, or pre-ride / in-ride UI.
---

# VeloSim UI parity

**Source of truth:** [docs/VeloSim-UI-and-Zwift-Parity-Guide.md](../../../docs/VeloSim-UI-and-Zwift-Parity-Guide.md)

Read the guide first. It overrides older navigation sketches and this skill's prior v1 TabView layout.

## Shell layout (current)

```
ContentView
├── shellPhase == .browse → AppShellView (Home · Activities · History · Settings)
└── shellPhase == .riding → RideModeView (Metal + Swift HUD + control bar)
```

Pre-ride setup lives on **Activities** (`PreRidePanel`), not a persistent sidebar.

## UI folder map

| Path | Purpose |
|------|---------|
| `UI/Design/` | `Tok`, `Typo`, `PowerZone` tokens |
| `UI/HUD/` | `RideHUDOverlay`, `RideHUDFormatting`, `HUDModel`, `HUDCoordinator` |
| `UI/Screens/` | Shell pages, `MetalRideView`, settings, summary |
| `UI/Components/` | `VeloGlass`, `HUDSurface` |

## HUD (single path)

- **Live ride:** Swift `RideHUDOverlay` on `MetalRideView`; values from throttled `HUDModel` (~8 Hz).
- **Rust glyphon HUD:** disabled at init (`setHudDrawEnabled(false)`); retained for screenshot capture only.
- **Never** fake glass with `.ultraThinMaterial` — use `hudSurface` / `.glassEffect` per guide §8.

## Compliance checklist (§11)

Before marking UI work done, verify every box in guide §11:

- [ ] No `.ultraThinMaterial`/custom blur; glass via `.glassEffect` or solid HUD fallback
- [ ] No glass-on-glass; no full-screen/content glass
- [ ] `GlassEffectContainer` per multi-element glass region
- [ ] Only power card tinted by zone
- [ ] HUD metrics match §5; layout matches §5.2
- [ ] HUD bound to ~8 Hz `HUDModel`, not tick/packet stream
- [ ] `.monospacedDigit()` + `.contentTransition(.numericText())` on numbers
- [ ] Accessibility: Reduce Transparency, Reduce Motion, Dynamic Type, VoiceOver

## Verification

```bash
cargo test --workspace
./scripts/lint-shell-ui.sh
./scripts/build.sh
```

## Superseded

- [docs/UI_PARITY_PLAN.md](../../../docs/UI_PARITY_PLAN.md) v1 TabView / Rust-HUD-primary plan — historical only.
