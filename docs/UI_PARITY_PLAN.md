# UI Parity Plan — VeloSim macOS Shell

**Tracking:** [#24](https://github.com/Gitter499/cyclosim/issues/24)

**Goal:** Replace the single-sidebar layout with a Zwift/MyWhoosh-style multi-destination app shell while preserving VeloSim architecture (Rust core, `velo-render` HUD, Liquid Glass chrome only).

**Skill:** [.cursor/skills/velo-ui-parity/SKILL.md](../.cursor/skills/velo-ui-parity/SKILL.md)

**Baseline:** `HSplitView` + `SetupChromeView` beside always-visible `MetalRideView` (`ContentView.swift`).

---

## Success criteria (P0)

- [ ] Browse mode shows **no persistent setup sidebar** and **no edge-docked Metal viewport**
- [ ] Four shell destinations: **Home**, **Activities**, **History**, **Settings**
- [ ] Ride mode is **full-bleed Metal** + Rust HUD + minimal stop/pause chrome only
- [x] Settings and API keys **unreachable** during active ride (except post-ride summary sheet)
- [ ] 3D Tiles toggle and status live on **Activities** (pre-ride), keys in **Settings**
- [ ] Existing ride flow tests pass; no FFI breaking changes without bindgen

---

## Phased delivery

### P0 — This sprint (functional parity)

| Work item | Description | Owner files |
|-----------|-------------|-------------|
| **P0-1 Shell router** | `AppShellView` + `ShellDestination` enum; top nav (text labels); `@Published shellPhase` on model | `UI/Shell/AppShellView.swift`, `UI/Shell/ShellDestination.swift`, `VeloSimModel.swift`, `ContentView.swift` |
| **P0-2 Dashboard home** | Last ride summary card, trainer connected badge, quick-start buttons (resume route / start free ride) | `UI/Shell/HomeDashboardView.swift`, `RideSummaryFormatting.swift`, `VeloSimModel.swift` |
| **P0-3 Activities catalog** | Route picker, GPX import, workout entry, ride mode, pre-ride trainer/music/steering panel, **3D tiles toggle + status** | `UI/Shell/ActivitiesCatalogView.swift`, extract from `SetupChromeView.swift` |
| **P0-4 History page** | Ride library list from `velo-rides` / `LocalRideStore` | `UI/Shell/RideHistoryView.swift`, `Ride/LocalRideStore.swift` |
| **P0-5 Settings flow** ✅ | Settings as nav destination (push or dedicated column); retain Keychain secrets | `UI/SettingsView.swift`, `AppSettingsStore.swift`, `AppSecretsStore.swift` |
| **P0-6 Ride screen layout** | `RideModeView`: full-window `MetalRideView`, floating stop bar, hide browse chrome | `UI/Shell/RideModeView.swift`, `ContentView.swift`, `VeloSimModel.swift` |
| **P0-7 HUD overlay basics** | Confirm `HudSnapshot` drives all in-ride metrics; remove Swift duplicate status for tiles in ride view | `crates/velo-render/src/hud.rs`, `crates/velo-ffi/src/lib.rs` (`hud_snapshot`) |
| **P0-8 Deprecate sidebar** | Delete or gut `SetupChromeView` after migration; keep `WorkoutBuilderView` reachable from Activities | `UI/SetupChromeView.swift` → remove |
| **P0-9 Tile loading checklist** | See § Tile loading fix checklist below | `velo-cesium`, `velo-render`, shell Activities |

**Suggested commit order:** P0-1 → P0-6 (empty stubs OK) → P0-3 → P0-4 → P0-2 → P0-5 → P0-7 → P0-8 → P0-9.

### P1 — Polish (next sprint)

| Work item | Description | Owner files |
|-----------|-------------|-------------|
| **P1-1 Nav animations** | Cross-fade / matched geometry on destination change; `@Namespace` glass morph per liquid-glass skill | `UI/Shell/*`, `VeloGlass.swift` |
| **P1-2 HUD layout** | Richer HUD: power zones, elevation strip, workout interval bar (MyWhoosh-inspired) | `crates/velo-render/src/hud.rs`, shaders if needed |
| **P1-3 HUD minimal mode** | Keyboard toggle (e.g. `U`) hiding HUD + chrome for screenshots | `VeloSimModel.swift`, `Input/SteeringInput.swift`, optional FFI flag |
| **P1-4 Home personalization** | Pin last route/workout on dashboard (My List lite) | `HomeDashboardView.swift`, `AppSettingsStore.swift` |
| **P1-5 Social placeholders** | Disabled “Events” / “Nearby” cards with copy — no backend | `ActivitiesCatalogView.swift` |
| **P1-6 Post-ride flow** | Ride summary as full-screen or large sheet from ride exit → Home | `RideSummarySheet.swift`, `RideModeView.swift` |
| **P1-7 Developer panel** ✅ | Rust log tail moved to Settings → Advanced | `SettingsView.swift` |

---

## File ownership matrix (implementation agent)

| Area | Primary owner | Do not touch without coordination |
|------|---------------|-----------------------------------|
| Shell navigation & pages | `shell-macos/Sources/VeloSim/UI/Shell/` | — |
| App state / ride phase | `VeloSimModel.swift` | `velo-ffi` if adding phase to FFI |
| Settings & secrets | `SettingsView.swift`, `AppSecretsStore.swift` | Keychain schema |
| Metal viewport host | `ContentView.swift` (`MetalRideView`) | `velo-render` |
| In-ride HUD | `crates/velo-render/src/hud.rs` | Shell should not draw metrics |
| HUD data assembly | `crates/velo-ffi/src/lib.rs` | `velo-core` session fields |
| 3D Tiles streaming | `crates/velo-cesium/src/session.rs` | Google ToS — online only |
| Tiles GPU draw | `crates/velo-render/src/tiles.rs` | Placeholder texture path |
| Workout builder UI | `WorkoutBuilderView.swift` | `velo-core` workout engine |
| Ride library | `velo-rides`, `LocalRideStore.swift` | SQLite schema |
| Tests | `shell-macos/Tests/VeloSimTests/` | Add `AppShellTests`, update `RideFlowTests` |
| Docs | `AGENTS.md` milestone row when P0 merges | This file |

---

## Architecture diagram (target)

```
┌──────────────────────────────────────────────────────────────┐
│ VeloSimApp                                                    │
│   ContentView                                                 │
│     ├─ shellPhase == .browse → AppShellView                   │
│     │     ├─ HomeDashboardView                                │
│     │     ├─ ActivitiesCatalogView  ← routes, workouts, tiles│
│     │     ├─ RideHistoryView                                  │
│     │     └─ SettingsView                                     │
│     └─ shellPhase == .riding → RideModeView                   │
│           ├─ MetalRideView (full bleed)                       │
│           ├─ velo-render HUD (via FFI render_frame)           │
│           └─ RideControlBar (Stop / optional Pause)           │
└──────────────────────────────────────────────────────────────┘
```

---

## Tile loading fix checklist (P0-9)

Known pain: 3D Tiles controls buried in sidebar; gray placeholder meshes; errors easy to miss.

### Shell

- [ ] **Move toggle** — `tiles3dEnabled` UI only on Activities route detail, not ride mode
- [ ] **Pre-ride validation** — show `tilesProviderStatus` + `tilesLastError` on route card before Start; block Start with clear message if keys missing and user enabled tiles
- [ ] **Settings-only keys** — Google / Cesium / Meshy remain in `SettingsView`; call `model.applySecrets()` after save before enabling tiles
- [ ] **Remove ride-sidebar noise** — no tile debug strings in Swift during ride; attribution stays in Rust HUD only

### FFI / model

- [ ] **`setRouteTiles3d`** called when user toggles on Activities **before** `startRide()`
- [ ] **Poll status** — refresh `tilesLastError()` on route select and every N sim ticks during pre-ride preview (optional lightweight preview)
- [ ] **`applySecretsToCore()`** before tile enable when keys change in Settings

### velo-cesium (`session.rs`)

- [ ] Verify `MissingApiKey`, `Offline`, network errors set retrievable last-error string
- [ ] Corridor updates on route progress — confirm `update_corridor` called from core during ride
- [ ] Dev ion fallback documented when no Google/Cesium key

### velo-render (`tiles.rs`, `lib.rs`)

- [ ] Document gray **placeholder texture** until real tile decode lands
- [ ] Ensure tile meshes rebuild when session delivers new `TileMesh` batches
- [ ] Attribution string passed into `HudSnapshot` only when provider active

### Manual test script

1. Settings → add Google key → save → apply
2. Activities → import/select route → enable 3D Tiles → confirm status OK
3. Start ride → full-screen → gray/colored tiles appear along route (or explicit error in pre-ride panel)
4. HUD shows attribution line; no Settings accessible until stop
5. Stop → summary → Home dashboard shows last ride

---

## Testing plan

| Test | Command / file |
|------|----------------|
| Shell navigation | New `AppScenarioTests` or extend `AppScenarioTests.swift` |
| Ride flow unchanged | `RideFlowTests.swift` |
| Settings Keychain | `AppSettingsStoreTests.swift`, `AppSecretsStoreTests.swift` |
| Rust workspace | `cargo test` |
| Apple symbol lint | `just lint` |
| Full app | `just build && just run` — manual P0 checklist above |

---

## Out of scope (explicit)

- Multiplayer, social graph, chat
- Avatar / garage / virtual currency
- Zwift Companion–style mobile app
- Rewriting `velo-render` as SwiftUI
- Offline 3D Tiles cache (ToS violation)

---

## References

Full URL list: [.cursor/skills/velo-ui-parity/reference.md](../.cursor/skills/velo-ui-parity/reference.md)
