# UI Parity P2 — Cohesive Shell, Settings, Activities, HUD

> **Authority:** [VeloSim-UI-and-Zwift-Parity-Guide.md](VeloSim-UI-and-Zwift-Parity-Guide.md) overrides this plan where they conflict.
> **Baseline:** M7 Zwift parity guide landed on `dev` via PR #37 (Closes #35).

**Goal:** Match Zwift / MyWhoosh *discipline* (not gamification) — cohesive browse shell, connection wizards, Activities catalog, and complete in-ride HUD per guide §5–§7.

---

## Platform study (what to copy vs skip)

| Area | Zwift | MyWhoosh | VeloSim target |
|------|-------|----------|----------------|
| **Browse nav** | Home cards, route/workout picker, profile hub | Calendar, route sorting, training plan | **System `TabView`** (Home · Activities · History · Settings) — glass from OS, not custom pills |
| **Settings** | In-ride HUD tab + hardware; profile prefs elsewhere | Equipment sliders (Gradient Feel), toggles | **Grouped list** (macOS Settings pattern): one row per service → **multi-step popover wizard** with live test |
| **Activities** | Route tiles, workout library w/ interval graph | Free ride UI, route filters | **Two-column**: catalog left, pre-ride summary right; 3D tiles validation on route card |
| **In-ride HUD** | Configurable 4-slot head unit + optional side stats panel | Unified HUD across modes, `U` minimal UI | Fixed zones §5.2; add **speed**, **elevation bar**, **rolling power**, **lap**, **ERG ±/skip** |
| **Connections** | Strava via Companion / web OAuth | Link app for social | **Strava wizard** (OAuth + test athlete API); **Apple Music wizard** (MusicKit auth + test search) |
| **Skip** | XP, drops, fake riders, events leaderboard | Prize money, ghost riders, chat | No stubs — out of scope per guide §6 |

### Zwift HUD specifics (2025–2026)

- **Configurable metrics** in workout vs free-ride profiles (13 biometrics in workout HUD).
- **Side stats panel** toggle: critical power windows + 2 extra slots.
- **Splits/laps** manual trigger → sidebar panel + FIT export.
- VeloSim v1: **fixed layout §5.2**; defer slot customization to P2-D.

### MyWhoosh specifics

- **Gradient Feel** (trainer resistance multiplier) — map to Settings → Equipment.
- **Minimal UI** (`U`) — already implemented; keep.
- **Virtual shifting** — out of scope until trainer protocol work.

---

## Design system (cohesion rules)

1. **Browse mode = plain content.** Window background + `.quaternary` cards. **No `VeloGlassSection` boxes on every block.**
2. **Glass only on:** floating HUD, ride control cluster, tab bar (system), wizard CTAs, primary buttons.
3. **Settings pattern:** `List` / `Form` with `.listStyle(.sidebar)` or inset grouped rows — **not** nested glass headers + glass bodies.
4. **Connection rows:** icon + title + status pill + chevron → opens **`.popover` or `.sheet`** wizard (3–4 steps).
5. **Activities pattern:** master–detail or single scroll with **section headers** (system), route cards with sparkline + distance/elev, pre-ride docked panel.

### Reference repos (Liquid Glass)

| Repo | Use |
|------|-----|
| [veersr9/Liquid-Glass-Reference](https://github.com/veersr9/Liquid-Glass-Reference) | `GlassEffectContainer`, morph IDs |
| [conorluddy/LiquidGlassReference](https://github.com/conorluddy/liquidglassreference) | TabView + `.tabBarMinimizeBehavior` |
| [sk1gl4a/LiquidGlass-SwiftUI-Showcase](https://github.com/sk1gl4a/LiquidGlass-SwiftUI-Showcase) | `.sidebarAdaptable`, sheets, popovers |
| Apple [Adopting Liquid Glass](https://developer.apple.com/documentation/TechnologyOverviews/adopting-liquid-glass) | **Use system TabView/toolbars — don't reskin** |

---

## Architecture target

```
ContentView
├─ shellPhase == .browse
│   └─ AppShellView (TabView, sidebarAdaptable)
│       ├─ Tab Home      → HomeDashboardView (keep — user likes it)
│       ├─ Tab Activities → ActivitiesCatalogView (master–detail)
│       ├─ Tab History   → RideHistoryView
│       └─ Tab Settings  → SettingsView (grouped list + wizards)
└─ shellPhase == .riding
    └─ RideModeView (Metal + RideHUDOverlay + controls)
```

---

## Phased delivery

### P2-A — Shell cohesion (this sprint)

| ID | Work | Files |
|----|------|-------|
| A1 | Replace custom `topNav` with **`TabView` + `Tab(role:)`** bound to `ShellDestination` | `AppShellView.swift`, `ShellDestination.swift` |
| A2 | Add `systemImage` per destination; `.tabViewStyle(.sidebarAdaptable)` on macOS | same |
| A3 | Remove redundant glass headers from browse pages (plain section titles) | `ActivitiesCatalogView`, `SettingsView`, `RideHistoryView` |
| A4 | **Settings → grouped `List`** with connection rows | `SettingsView.swift`, new `UI/Settings/` |
| A5 | **`StravaConnectionWizard`** popover: (1) status (2) connect OAuth (3) test `/athlete` (4) done | `StravaAuthCoordinator`, new wizard view |
| A6 | **`AppleMusicConnectionWizard`** popover: (1) intro (2) MusicKit auth (3) test catalog search (4) done | `VeloMusicDirector`, new wizard view |
| A7 | **Activities master–detail**: route list + detail (tiles toggle, sparkline, Start) | `ActivitiesCatalogView.swift`, `ParityHelpers.swift` |
| A8 | Move API keys (Google/Cesium/Meshy) to **Settings → Integrations** sub-wizard | `SettingsView`, `AppSecretsStore` |

**Acceptance:** Settings has ≤1 glass element per screen; Strava/Music wizards can prove connectivity without starting a ride.

### P2-B — HUD feature parity (guide §5 + §7.5)

| ID | Work | Files |
|----|------|-------|
| B1 | Add **SPEED** to secondary cluster (kph/mph from prefs) | `HUDModel`, `RideHUDOverlay`, FFI if needed |
| B2 | **Route/elevation profile bar** bottom (position dot, gradient fill) | `RideHUDOverlay`, route FFI samples |
| B3 | **Rolling power graph** (~60 s ring buffer in HUDModel) | `HUDModel`, `HUDCoordinator`, overlay |
| B4 | **Lap** button + lap time in top pill | `VeloSimModel`, overlay |
| B5 | Workout bar: **ERG bias ±**, **skip interval**, next block name | `WorkoutBarView`, core workout API |
| B6 | **Pause menu** glass card (Resume / End / Discard) — verify against guide §7.6 | `RideModeView`, `ParityHelpers` |
| B7 | Post-ride summary fields: NP, TSS, IF, elevation gain | `RideSummarySheet`, core aggregates |

### P2-C — Activities depth (guide §7.3–7.4)

| ID | Work |
|----|------|
| C1 | Workout library: interval graph thumbnails + TSS estimate |
| C2 | Route cards: real elevation polyline from `RouteModel` |
| C3 | Pre-ride validation banner (tiles keys, trainer, music) |
| C4 | FTP test picker polish (ramp + 20-min protocols complete) |

### P2-D — Polish & Zwift-advanced (later)

| ID | Work |
|----|------|
| D1 | HUD metric slot customization (free vs workout profiles) |
| D2 | Side stats panel (CP windows) — toggle from action bar |
| D3 | Gradient Feel slider (Settings → Equipment) |
| D4 | Home fitness trends charts (Companion-style, local SQLite) |

---

## Settings information architecture (target)

```
Settings (List)
├── Profile          → FTP, weight, name (inline)
├── Connections
│   ├── Strava       → wizard popover
│   ├── Apple Music  → wizard popover
│   └── Trainer      → link to Activities pairing sheet
├── Integrations
│   ├── 3D Tiles     → Google / Cesium key wizard + test ping
│   └── Bikegen      → Meshy key + mode
├── Ride defaults    → steering, music toggle, units
└── Advanced         → developer (collapsed)
```

**Anti-pattern removed:** one giant `VeloGlassSection` per topic with nested dividers and duplicate status badges.

---

## Connection wizard pattern (shared)

```swift
enum WizardStep: Int, CaseIterable { case intro, action, test, done }

struct ConnectionWizard<Content: View>: View {
    @Binding var step: WizardStep
    let title: String
    let testAction: () async -> WizardTestResult
    @ViewBuilder var body: () -> Content
}
```

- **Test step** calls a real API (Strava `/athlete`, MusicKit catalog search, tiles status poll).
- **Done** shows green check + "Connected" status persisted to model.
- Use `.buttonStyle(.glassProminent)` only on primary wizard CTAs.

---

## HUD parity matrix (guide §11 extended)

| Metric / control | Guide | PR #37 | P2-B |
|------------------|-------|--------|------|
| Power (zone tint) | ✅ | ✅ | — |
| Cadence, HR, w/kg | ✅ | ✅ | — |
| Speed | ✅ | ❌ | B1 |
| Time, dist, grade | ✅ | ✅ | — |
| Elevation profile | ✅ | stub | B2 |
| Rolling power | ✅ | ❌ | B3 |
| Workout bar | ✅ | partial | B5 |
| Lap | ✅ | ❌ | B4 |
| Ride controls | ✅ | ✅ | verify |
| Minimal UI (U) | ✅ | ✅ | — |

---

## Testing

| Area | Test |
|------|------|
| Tab navigation | `AppScenarioTests` — destination binding |
| Wizards | `StravaOAuthTests` + new `ConnectionWizardTests` (mock HTTP) |
| HUD buffer | `HUDCoordinatorTests` rolling power samples |
| Lint | `./scripts/lint-shell-ui.sh` (no ultraThinMaterial in HUD) |
| Manual | §11 checklist + wizard test flows |

---

## GitHub tracking

| Issue | Scope |
|-------|-------|
| #38 (proposed) | P2-A Shell + Settings wizards + Activities layout |
| #39 (proposed) | P2-B HUD feature parity |
| #40 (proposed) | P2-C Activities depth + FTP protocols |

---

## Out of scope (unchanged)

Multiplayer, avatars, XP/drops, offline tile cache, HUD slot customization (until P2-D).
