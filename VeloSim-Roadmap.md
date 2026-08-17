# VeloSim — Product Roadmap & Integration Plan

> **Companion doc:** [VeloSim-Technical-Plan.md](VeloSim-Technical-Plan.md) (architecture, crates, testing, quality log).  
> **Agents:** UI work must follow **Part II** below. Skills point here.

Last updated: 2026-07-01 · branch `dev`

---

## Part I — Integration plan

### Vision

Solo macOS cycling simulator with **Zwift / MyWhoosh discipline** (dense HUD, structured workouts, real routes) **without** multiplayer gamification (no XP, drops, fake riders, leaderboards).

### Current state (`dev`)

| Area | Status |
|------|--------|
| Rust core, trainer, FIT, Strava, ride library | ✅ M0–M2c |
| Routes, terrain, 3D Tiles spike, bike import | ✅ M3–M3c |
| Workouts, `.zwo`, highlight clips | ✅ M5 |
| MusicKit + steering | ✅ M6 (playback polish open) |
| HUD §5.2 skeleton, browse shell | ✅ M7 partial (#37, #39, #46) |
| P2-A UI (TabView, Settings wizards, Activities master–detail) | ✅ merged |
| Browse scroll performance | ✅ sim loop idle in browse (#46) |
| BLE heart rate to HUD | ✅ #46 (connection **UI feedback** still missing) |
| Full HUD parity (speed, elev bar, laps, ERG ±) | ❌ P2-B |
| Real route elevation / workout graphs | ❌ P2-C |
| Gaussian splats (M4) | ❌ #9 |
| Codebase cleanup (ParityHelpers split) | 🚧 partial #42 |

### Platform parity matrix

| Area | Zwift | MyWhoosh | VeloSim now | Target |
|------|-------|----------|-------------|--------|
| Browse nav | Home hub, cards | Calendar, route sort | Sidebar TabView | ✅ keep |
| Settings | In-ride HUD tab + hardware | Gradient Feel, equipment | Grouped list + wizards | Equipment + HUD prefs |
| Activities | Route tiles, workout graphs | Free ride filters | Master–detail, stub sparklines | Real elev + interval graphs |
| In-ride HUD | Configurable 4-slot + side panel | Unified HUD, `U` minimal | Zone power, workout bar partial | §5.2 complete + optional slots (later) |
| Pairing | Device rows + status | Trainer connect | Pairing sheet | **Connected state + HR row** |
| Connections | Strava Companion | Link app | Strava/Music wizards | ✅ + BLE status |
| Skip | XP, drops, events | Ghost riders, chat | — | Never |

### Phased delivery

#### Phase 1 — Polish & feedback (next)

| ID | Work | Issue |
|----|------|-------|
| 1.1 | **BLE pairing UX:** show HR/trainer/cadence connected state, battery, live BPM preview | [#47](https://github.com/Gitter499/cyclosim/issues/47) |
| 1.2 | **MusicKit segment playback** reliability | [#29](https://github.com/Gitter499/cyclosim/issues/29) |
| 1.3 | Quality pass: split `ParityHelpers` into focused screen files | [#42](https://github.com/Gitter499/cyclosim/issues/42) |

#### Phase 2 — HUD feature parity (P2-B)

| ID | Work | Issue |
|----|------|-------|
| 2.1 | Speed metric in HUD | [#48](https://github.com/Gitter499/cyclosim/issues/48) |
| 2.2 | Route/elevation profile bar + position dot | #48 |
| 2.3 | Rolling power graph (~60 s) | #48 |
| 2.4 | Lap button + lap time | #48 |
| 2.5 | Workout bar: ERG bias ±, skip interval, next block | #48 |
| 2.6 | Post-ride summary: NP, TSS, IF, elevation gain | #48 |

#### Phase 3 — Activities depth (P2-C)

| ID | Work | Issue |
|----|------|-------|
| 3.1 | Workout library interval graph thumbnails + TSS | [#49](https://github.com/Gitter499/cyclosim/issues/49) |
| 3.2 | Real route elevation from `RouteModel` | #49 |
| 3.3 | Complete 20-min FTP protocol | #49 |
| 3.4 | Pre-ride validation banner (tiles, trainer, music) | #49 |

#### Phase 4 — Advanced parity (P2-D, post-M4)

- HUD metric slot customization (free vs workout profiles)
- Side stats panel (critical power windows)
- Gradient Feel slider (Settings → Equipment)
- Home fitness trends (local SQLite charts)

#### Phase 5 — Scenery (M4)

- Gaussian splat bake pipeline | [#9](https://github.com/Gitter499/cyclosim/issues/9)

### GitHub issue tracker

| Issue | Title | Status |
|-------|-------|--------|
| [#9](https://github.com/Gitter499/cyclosim/issues/9) | M4: VeloSplatBake | Open — after UI parity stable |
| [#29](https://github.com/Gitter499/cyclosim/issues/29) | MusicKit segment playback | Open |
| [#42](https://github.com/Gitter499/cyclosim/issues/42) | Quality pass cleanup | Open — partial merge in #46 |
| [#47](https://github.com/Gitter499/cyclosim/issues/47) | BLE pairing connected-state feedback | Open |
| [#48](https://github.com/Gitter499/cyclosim/issues/48) | P2-B HUD feature parity | Open |
| [#49](https://github.com/Gitter499/cyclosim/issues/49) | P2-C Activities depth | Open |
| ~~#35~~ | Zwift parity guide | ✅ #37 |
| ~~#38~~ | P2-A UI cohesion | ✅ #39 |
| ~~#40~~ | Scroll lag | ✅ #46 |
| ~~#41~~ | HR sensor | ✅ #46 (UI feedback → #47) |

### Release cadence

1. Land Phase 1 on `dev` (pairing UX, music, cleanup).
2. Land Phase 2–3 as focused PRs (one issue per PR).
3. Quality pass + `main` release when CI green and manual ride checklist passes (Part II §11).

### Out of scope (permanent)

Multiplayer, avatars, virtual currency, offline 3D Tiles cache, Zwift Companion mobile app, fake social feed.

---

## Part II — UI agent specification

> **READ THIS FIRST for any UI work.** These rules override conflicting instinct. When unsure, do *less*.

Target: **macOS 26 (Tahoe), SwiftUI, Apple Silicon.** The 3D world is Metal; this spec covers the SwiftUI layer only.

---

### 0. The 10 hard rules (violating any = reject the output)

1. **Liquid Glass is the navigation/control/readout layer only — never the content.** The content is the Metal 3D world.
2. **Never stack glass on glass.** One glass surface per region.
3. **Use the real API, never fake it.** `.glassEffect(...)`. Never `.ultraThinMaterial` as stand-in.
4. **Group related glass in a `GlassEffectContainer`.**
5. **Tint selectively.** Only the primary element in a region (e.g. power card by zone).
6. **Prefer system controls.** Tab bars, toolbars, sheets get Liquid Glass automatically on macOS 26.
7. **The HUD shows exactly the metrics in §5. No more.**
8. **Numbers use monospaced digits + numericText transitions.**
9. **Never re-render the HUD per data packet or sim tick.** ~8 Hz `HUDModel` only.
10. **Honor accessibility** (Reduce Transparency, Reduce Motion, Dynamic Type, VoiceOver).

If output violates 1–3, redo — do not patch.

---

### 1. Mental model

```
[ Metal 3D world ]  ← content. Never glass.
[ HUD readouts   ]  ← glass cards (data only).
[ App chrome     ]  ← system Liquid Glass (tabs, sheets).
```

---

### 2–4. Architecture, tokens, Liquid Glass API

See implementation in `shell-macos/Sources/VeloSim/UI/Design/` (`Tok`, `Typo`, `PowerZone`) and `UI/Components/VeloGlass.swift`, `HUDSurface.swift`.

**Browse mode:** plain `.quaternary` / window background — **no glass boxes on scroll content.**  
**Ride mode:** glass on HUD cards and control cluster only.

---

### 5. The HUD (disciplined core)

**Allowed metrics (exhaustive):** Power, Cadence, HR, Speed, Distance, Time, w/kg, Gradient.  
Contextual: Lap, workout target vs actual, route/elevation profile, rolling power (~60 s).

**Layout zones (fixed):** top pill (time · dist · grade) → bottom-left power + secondary stats → bottom-right controls → optional workout/elevation bar.

**Update cadence:** ~8 Hz via `HUDCoordinator` → `HUDModel`. Never bind browse views to `rideState` at sim tick rate.

---

### 6. Zwift feature parity (solo)

- Ride modes: Just Ride, Workout (ERG), FTP Test (ramp + 20-min protocols in `FTPTestEngine`).
- Workout blocks: warmup, steady, intervals, ramp, freeRide, cooldown — targets as % FTP.
- No multiplayer stubs.

---

### 7. Screens

| Screen | Spec |
|--------|------|
| Home | Profile, quick-start row, recent rides, lifetime stats — plain content |
| Pairing | Per-device rows with **connected state**; glass Ride CTA |
| Activities | Master–detail; route sparkline; tiles toggle; pre-ride panel |
| Settings | Grouped list; connection wizards (Strava, Music, Integrations) |
| In-ride | HUD §5 + workout bar + pause menu |
| Summary | Plain cards; glass only on CTAs |

---

### 8–9. Accessibility & performance

- `hudSurface()` fallback when Reduce Transparency is on.
- Sim loop **off in browse**; only runs during rides.
- One `GlassEffectContainer` per HUD region; animate tint/values only.

---

### 10. Anti-patterns (reject on sight)

Glass on every card, glass-on-glass, fake blur, rainbow tints, loose glass without container, HUD bound to tick stream, invented metrics, fake riders/XP, reskinned system tab bar, centered wall of numbers.

---

### 11. Compliance checklist (agent self-verify before "done")

- [ ] No `.ultraThinMaterial`/custom blur; real `.glassEffect` only
- [ ] No glass-on-glass; no full-screen content glass
- [ ] Multi-element glass in `GlassEffectContainer`
- [ ] Only power card zone-tinted
- [ ] HUD metrics ⊆ §5; layout matches §5.2
- [ ] HUD ~8 Hz model, not tick stream
- [ ] `.monospacedDigit()` + `.contentTransition(.numericText())`
- [ ] FTP tests use exact §6.3 protocols
- [ ] Workout targets %FTP → watts; freeRide = SIM not ERG
- [ ] Accessibility handled
- [ ] System nav/sheets not reskinned
- [ ] No multiplayer/gamification stubs

---

*Full code samples and FTP formulas were in the prior standalone guide; implementation lives in `shell-macos/Sources/VeloSim/UI/`.*
