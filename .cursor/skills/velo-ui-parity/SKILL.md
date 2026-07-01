---
name: velo-ui-parity
description: >-
  VeloSim shell navigation parity (Zwift/MyWhoosh v1): multi-page TabView,
  dashboard quick starts, fullscreen ride + Swift HUD overlay, dedicated Settings.
  Use when restructuring shell chrome or adding pre-ride / in-ride UI.
---

# VeloSim UI parity v1

## Navigation

```
TabView
├── Home (DashboardView) — quick start cards, recent rides
├── Activities — Routes | Workouts | History
├── Ride (RideView) — full-bleed Metal + RideHUDOverlay
└── Settings (SettingsView) — Keychain secrets, preferences
```

Pre-ride BLE / steering / music / ERG → **PreRideSetupSheet** from Ride tab, not a permanent sidebar.

## HUD

- **Live ride:** Swift `RideHUDOverlay` on `MetalRideView` (corners + workout banner + tiles attribution in status bar).
- **Rust glyphon HUD:** disabled via `setHudDrawEnabled(false)` after renderer init; still drawn on screenshot capture.
- Formatting: `RideHUDFormatting.swift` (tested).

## Tiles

- Google path-only URIs resolve against `https://tile.googleapis.com` with `session` + `key`.
- Errors surface on Ride tab status bar (`tilesLastError`, `tilesProviderStatus`).

## P1 deferrals

- Gaussian splatting scenery
- Hosted Meshy bikegen HTTP
- Deep tile LOD streaming / frustum culling
- Morphing Liquid Glass chrome between tabs

## Verification

```bash
cargo test --workspace
./scripts/lint-shell-ui.sh
./scripts/build.sh
```
