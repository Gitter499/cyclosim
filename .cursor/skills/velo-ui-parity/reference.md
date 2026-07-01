# VeloSim UI parity — research references

Sources used to define mandatory patterns in [SKILL.md](SKILL.md). Accessed June 2025 unless noted.

---

## Official / primary sources

### Zwift

| Topic | Source | URL | Takeaway for VeloSim |
|-------|--------|-----|----------------------|
| Home screen IA (2024) | Zwift Forums — Shuji (staff) | https://forums.zwift.com/t/my-list-and-home-screen-updates-april-2024/629572 | Multi-tab home: **Home, Events, Routes, Workouts**; text nav; horizontal card rows with edge fade; home optimized for **quick ride entry**; planning on separate pages |
| Navigation bar (2023) | Zwift Forums — Charlie_CC (staff) | https://forums.zwift.com/t/navigation-bar-updates-feb-2023/602928 | **Text labels over icons**; profile dropdown for settings/achievements/garage; challenges as top-level tab; reduce home clutter |
| My List / For You | Zwift Insider | https://zwiftinsider.com/my-list/ | Planned activities surface on home **For You** carousel; catalog browsing separate from home |
| Activity cards / Join | Zwift Insider v1.101 | https://zwiftinsider.com/update-1-101-154518/ | Wider activity cards; **Join a Zwifter** promoted to top nav |
| Home scaling removed | Zwift Insider | https://zwiftinsider.com/home-screen-scaling/ | Single standard scale across devices — VeloSim: one macOS layout, optional window size only |

### MyWhoosh

| Topic | Source | URL | Takeaway for VeloSim |
|-------|--------|-----|----------------------|
| Home + modes | Cyclist review | https://www.cyclist.co.uk/reviews/mywhoosh-indoor-cycling-app-review | Launch → **home with profile, settings, currency, ride options**; modes: free ride, workouts, events |
| HUD layout | Cyclist review | https://www.cyclist.co.uk/reviews/mywhoosh-indoor-cycling-app-review | **Top**: time, HR, power, cadence, speed, distance; **bottom**: avg speed, kcal, elevation; **right**: nearby list + route map/gradient |
| HUD 4.0 redesign | MyWhoosh blog | https://mywhoosh.com/new-features-more-control-and-the-best-indoor-cycling-yet-mywhoosh-4-0-0/ | Unified HUD across Free Ride / Workouts / Events; more metrics at a glance |
| HUD critique (layout) | the5krunner | https://the5krunner.com/2025/03/12/mywhoosh-4-major-upgrade/ | Top-center primary metrics; map/elevation corner; segment bars for workouts — informs P1 HUD layout |
| Minimal UI / hide HUD | MyWhooshInfo | https://mywhooshinfo.com/blog/mywhoosh-keyboard-shortcuts | **U** = minimal UI; **H** = hide all controls — P1 parity for screenshots/focus |
| Link companion | Cyclist review | https://www.cyclist.co.uk/reviews/mywhoosh-indoor-cycling-app-review | External app adjusts ride settings **without** stuffing main menu — VeloSim defers companion; use Settings + pre-ride only |

---

## UX / design articles

| Source | URL | Relevance |
|--------|-----|-----------|
| Cloning Zwift — SwiftUI workout UI | https://www.hung-truong.com/blog/2021/05/02/zwift-clone-swiftui-combine/ | Workout segment visualization as distinct **detail view**; GroupBox separation — applies to Activities workout picker, not ride HUD |
| Zswift POC (SwiftUI) | https://github.com/hungtruong/Zswift | Early SwiftUI shell split from game view |

---

## Open-source UI references (patterns only)

Not visual clones — architectural hints.

| Project | Stars | URL | Relevant pattern |
|---------|-------|-----|------------------|
| **Auuki** | ~850+ | https://github.com/dvmarinoff/Auuki | PWA: workout picker → full-screen ride; Web Components chrome vs canvas |
| **GlobeRide** | new | https://github.com/masonwyatt23/globeride | React + Cesium: route picker, workout calendar, post-ride analytics as **separate views**; Zustand app phase |
| **zwift-app** | small | https://github.com/taehoio/zwift-app | Expo Router file-based **Home → Event detail** navigation |

---

## VeloSim internal docs

| Doc | Path | Relevance |
|-----|------|-----------|
| Technical plan | `VeloSim-Technical-Plan.md` | Tier B tiles online-only; wgpu HUD; Swift owns chrome only |
| Liquid Glass skill | `.cursor/skills/liquid-glass/SKILL.md` | Glass on nav/sheets only |
| AGENTS.md | `AGENTS.md` | Milestone status, crate boundaries |

---

## Current VeloSim baseline (pre-parity)

| File | Role |
|------|------|
| `shell-macos/Sources/VeloSim/ContentView.swift` | `HSplitView`: Metal + sidebar — **to replace** |
| `shell-macos/Sources/VeloSim/UI/SetupChromeView.swift` | Monolithic setup sidebar — **to split** |
| `shell-macos/Sources/VeloSim/UI/SettingsView.swift` | Settings sheet — **keep**, wire to Settings destination |
| `crates/velo-render/src/hud.rs` | In-ride HUD text overlay |
| `crates/velo-ffi/src/lib.rs` | `hud_snapshot()`, render_frame |
| `crates/velo-cesium/src/session.rs` | Tile session errors → `tilesLastError()` |

---

## Deferred vs v1 (explicit)

| Feature | Competitor | VeloSim v1 | Target |
|---------|------------|------------|--------|
| Social / nearby riders | Both | Defer | P1 placeholder or hide |
| Avatar / garage | Zwift | Defer | Post-v1 |
| Events calendar | Both | Defer | P1 stub on Activities |
| Companion / Link app | Both | Defer | Not planned |
| Customizable home rows | Zwift For You / My List | Defer | P1: pin last route/workout |
| HUD minimal toggle | MyWhoosh U/H | Defer | P1 via keyboard |
| Zone-colored power HUD | Both | Defer | P1 velo-render |
| Elevation profile / ClimbPro map | Both | Defer | P1 velo-render |
| In-ride chat / reactions | MyWhoosh | Defer | Out of scope |
| Multiplayer | Zwift | Out of scope | — |
| Training plans marketplace | Both | Defer | M5+ builder sufficient for v1 |

---

## Browser research note

Marketing sites (zwift.com, mywhoosh.com) emphasize hero video, not in-app IA. Primary evidence comes from **official forum posts**, **MyWhoosh release notes**, and **third-party reviews** above. App Store screenshots were not required for P0 IA decisions.
