# Quality pass log

Running record of cross-cutting cleanup on the `dev` branch. See [.cursor/skills/quality-pass/SKILL.md](../.cursor/skills/quality-pass/SKILL.md) for the agent workflow.

| Date | Trigger | Summary | Commits |
|------|---------|---------|---------|
| 2026-06-30 | UI cleanup prep (#34) | Anti-pattern removal, UI folder restructure, HUDModel ~8 Hz skeleton | see PR |

---

# Quality pass — 2026-06-30 (UI cleanup prep #34)

## Scope
Milestone / trigger: **UI cleanup prep** before implementing `docs/VeloSim-UI-and-Zwift-Parity-Guide.md`.

## Changes made
- Restructured `shell-macos/Sources/VeloSim/UI/` into `Design/`, `HUD/`, `Screens/`, `Components/`.
- Removed `.ultraThinMaterial` fake-glass fallbacks; added `HUDSurface` (guide §8) and solid `.quaternary` chrome fallback.
- Documented single HUD path: Swift overlay live, Rust glyphon disabled at init.
- Added `HUDModel` + `HUDCoordinator` (~8 Hz throttling skeleton).
- Added `PowerZone`, `Tok`, `Typo` design tokens; `PowerZoneTests`, `HUDCoordinatorTests`.
- Updated `velo-ui-parity` skill, `UI_PARITY_PLAN.md` (superseded notice), `lint-shell-ui.sh` (flags `.ultraThinMaterial` in HUD/Components).

## Findings (deferred — for implementation agent)
- **HUD layout rewrite** — current overlay is MyWhoosh-inspired placeholders; rebuild to guide §5.2 zones with real glass cards.
- **Power zone tint** — `PowerZone` exists but overlay does not tint the power card yet.
- **Rider weight / w/kg** — `HUDCoordinator` accepts weight but shell has no profile weight field yet.
- **Chrome glass** — pre-macOS-26 uses solid `.quaternary`, not glass; full Liquid Glass requires macOS 26 SDK locally.
- **Pairing screen, FTP tests, dashboard §7 layouts** — explicitly out of scope for this prep PR.

## Doc sync
- `velo-ui-parity` skill now points to parity guide §11 checklist.
- `UI_PARITY_PLAN.md` marked superseded; architecture diagram updated for Swift HUD path.

## Test coverage added
- `PowerZoneTests.swift` — Coggan zone boundaries.
- `HUDCoordinatorTests.swift` — 8 Hz throttle (single write per burst).

## Verification
- `cargo test --workspace`
- `./scripts/lint-apple-symbols.sh`
- `./scripts/lint-shell-ui.sh`
- `./scripts/build.sh`

---
| 2026-06-30 | Post-M5 complete (#10 closed) | Agent skills (rust/swift best practices); M5 doc sync; quality-pass skill cross-ref | see below |
| 2026-06-30 | Initial bootstrap | Doc sync (plan M2c done, workspace layout); FFI callback + FIT catalog integration tests; test-helper warning cleanup | `a1b2c3d`… (see git log before this pass) |
| 2026-06-30 | Post-M5 partial + multi-agent desync | Plan/README sync (M3b/M3c done, M5 partial); unified `RecordingTrainerControl`; doc refresh | see below |
| 2026-06-30 | Post-M5 slices (#16, #17) | M5 doc sync; FFI test common mocks; highlight golden + schema v2 migration tests | see below |

---

# Quality pass — 2026-06-30 (pre headless app testing)

## Scope
Milestone / trigger: **Pre headless app testing** — M6 on `feat/issue-11-m6-music-steering`; expand scenario tests before Kickr hardware-in-the-loop.

## Changes made
- Added user-story integration tests: `velo-core/tests/scenarios/`, `steering_golden.rs`, `audio_segment.rs`, `route_import_scenario.rs`.
- Added `velo-ffi/tests/app_scenarios.rs`; extended `tests/common/mod.rs` with `ReplaySensors`, `fixture_gpx_path`, recording mocks.
- Extracted `WorkoutBuilderInterval` to `VeloSimSupport` (`Workout/`) for testable DTO mapping; executable excludes `Workout/` (same pattern as `Ride/`, `Music/`).
- Added `shell-macos/Tests/VeloSimTests/AppScenarioTests.swift`.
- New guide: [docs/HEADLESS_TESTING.md](HEADLESS_TESTING.md).

## Findings (deferred)
- **Swift CI** — `swift test` not in default GitHub Actions job; local Xcode + `libvelo_ffi.dylib` required.
- **Hardware-in-the-loop** — Kickr ERG hold, SIM grade response, BLE dropout: manual checklist placeholder in HEADLESS_TESTING.md.
- **Cinematic replay camera** for highlight clips — unchanged from prior passes.

## Doc sync
- `velo-core`, `velo-ffi` README test tables updated for M6 + scenario files.
- `velo-ffi` README milestone row includes M6 callbacks.
- Plan §19 references headless testing doc.

## Test coverage added
- See [HEADLESS_TESTING.md](HEADLESS_TESTING.md) test map.

## Verification
- `cargo test --workspace`
- `./scripts/lint-apple-symbols.sh`
- `./scripts/lint-shell-ui.sh`
- `cd shell-macos && swift build --product VeloSim` (local)

---

## Scope
Milestone / trigger: **M5 complete** — issue #10 closed; PR #18 merged (`.zwo` import); full pass using quality-pass + rust-best-practices + swift-best-practices skills before `dev` → `main` release.

## Changes made
- Added `.cursor/skills/rust-best-practices/` and `.cursor/skills/swift-best-practices/` agent checklists with optional `reference.md`.
- Extended quality-pass skill to cross-reference rust + swift skills in checklist.
- Synced M5 → ✅ across plan workspace line, `velo-core`, `velo-ffi`, `velo-rides`, and `shell-macos` READMEs.
- Updated `AGENTS.md` skill links.

## Findings (deferred)
- **Cinematic replay camera** for highlight clips — ring-buffer + VideoToolbox encode ships; replay-camera path remains future work.
- **Apple-symbol lint** still scans only `velo-core`, `velo-units`, `velo-platform`.
- **CI** runs on `main`/`master` PRs — gate for release; `dev` merges skip CI wait per workflow.
- **Package.swift** `Ride/` exclude on executable target is intentional (compiled via `VeloSimSupport`).

## Doc sync
- Plan §3 workspace header: M5 complete (was "M5 partial").
- Crate README milestone rows updated from "M5 partial" to "M5".
- Shell README documents M5 Liquid Glass, builder, `.zwo`, highlight clips.

## Test coverage added
- No new tests this pass — existing M5 coverage retained (`workout_integration`, `zwo_import`, `highlight_clips`, `HighlightClipEncoderTests`).

## Verification
- `cargo test --workspace`
- `./scripts/lint-apple-symbols.sh`
- `./scripts/lint-shell-ui.sh`

---

# Quality pass — 2026-06-30 (post-M5 slices)

## Scope
Milestone / trigger: **Post-M5 slices merged to `dev`** — PR #16 (Liquid Glass setup/summary), PR #17 (highlight clips); workout builder already on `dev`.

## Changes made
- Synced `VeloSim-Technical-Plan.md` M5 section: builder, Liquid Glass, highlight clips marked shipped; `.zwo` import and cinematic replay camera deferred.
- Updated `AGENTS.md` M5 row; refreshed `velo-core`, `velo-ffi`, `velo-rides` READMEs (`highlight` module, `MediaCaptureCallback::encode_highlight_clip`, schema v2).
- Unified FFI integration test mocks in `velo-ffi/tests/common/mod.rs` (`MockMedia`, `MockPublisher`, `RecordingTrainerCallback`, `NoopTrainer`, `TickSensors`).
- Added `velo-core/tests/highlight_clips.rs` golden test for `plan_highlight_clips`.
- Added `velo-rides` v1→v2 migration test preserving existing rows and `highlight_clip_path` updates.

## Findings (deferred)
- **Cinematic replay camera** for highlight clips — ring-buffer capture ships; replay-camera path remains future work.
- **`.zwo` import** — last open M5 acceptance item (#10).
- **Apple-symbol lint** still scans only `velo-core`, `velo-units`, `velo-platform`.
- **CI** runs on `main`/`master` only — acceptable per rapid-dev workflow on `dev`.
- **Package.swift** `Ride/` exclude on executable target is intentional (compiled via `VeloSimSupport`); no change needed.

## Doc sync
- Plan §13 highlight-clip bullets updated to match shipped ring-buffer + VideoToolbox path.
- Plan §14b ride row list includes `highlight_clip_path`.
- FFI README callback table matches generated UniFFI surface.

## Test coverage added
- `velo-core/tests/highlight_clips.rs` — golden windows for 120 s ride + short ride.
- `velo-rides/tests/migration.rs` — `migrate_v1_db_adds_highlight_clip_path`.
- Existing `velo-ffi/tests/ride_library_integration.rs::finish_ride_plans_and_encodes_highlight_clips` retained.

## Verification
- `cargo test --workspace`
- `./scripts/lint-apple-symbols.sh`
- `./scripts/lint-shell-ui.sh`

---

# Quality pass — 2026-06-30 (post-M5 partial)

## Scope
Milestone / trigger: **Post-M5 partial + multi-agent desync** — `velo-bikegen`, `velo-cesium`, workout FFI/HUD/shell landed in parallel; plan and test helpers had drifted.

## Changes made
- Synced `VeloSim-Technical-Plan.md`: workspace lists `velo-cesium` + `velo-bikegen`; M3b/M3c marked done; M5 marked partial with shipped vs remaining bullets; removed stale "HUD state" from core architecture diagram.
- Added `RecordingTrainerControl` in `velo-platform` (mutex-backed ERG/SIM recording); simplified `MockTrainerControl` to a true no-op.
- Replaced duplicate local `RecordingTrainer` / `GradeRecordingTrainer` stubs in `velo-core` tests with shared `RecordingTrainerControl`.
- Updated `velo-core`, `velo-ffi`, and `velo-platform` READMEs (workout module, integration test inventory, M5 partial status).

## Findings (deferred)
- **FFI test stubs** — addressed in post-M5 slices pass via `tests/common/mod.rs`.
- **Apple-symbol lint** still scans only `velo-core`, `velo-units`, `velo-platform`; extend to other portable crates when convenient.
- **M5 remaining** — `.zwo` import, cinematic replay camera (#10).
- **Cesium Native C++ bridge** — Rust glTF path ships; full `cxx` + CMake linking deferred per `velo-cesium` README.
- **CI** runs on `main`/`master` only — acceptable per rapid-dev workflow on `dev`.

## Doc sync
- `AGENTS.md` milestone table already current (M3c ✅, M5 🔜).
- Root `README.md` intentionally minimal; crate/milestone tables live in `AGENTS.md`.
- Crate READMEs for M3 crates (`velo-route-import`, `velo-terrain`, `velo-cesium`, `velo-bikegen`) reviewed — no drift found.

## Test coverage added
- `velo-platform`: unit test for `RecordingTrainerControl` command capture.
- Existing integration coverage retained: `workout_erg`, `workout_integration`, `bike_integration`, `tiles_integration` (no new cross-crate files this pass).

## Verification
- `cargo test --workspace` — all tests pass.
- `./scripts/lint-apple-symbols.sh` — passed.

---

# Quality pass — 2026-06-30 (initial bootstrap)

## Scope
Milestone / trigger: **Initial quality pass bootstrap** on `dev` after M0–M2c monolith import.

## Changes made
- Marked **M2c** as done in `VeloSim-Technical-Plan.md`; clarified which crates exist in the workspace vs planned (M3+).
- Fixed `velo-core` crate description (removed stale "HUD state" — HUD lives in `velo-render`).
- Added FFI integration tests for sensor → `ride_state` and ERG/SIM trainer callback forwarding.
- Added cross-crate integration test: `velo-fit` encode → `velo-rides` artifact save → on-disk FIT parse.
- Silenced `dead_code` warnings in shared `velo-rides` test helpers used across integration test binaries.

## Findings (deferred)
- See post-M5 slices pass above for current deferred list; items below were addressed or superseded.
- **`velo-core` `ride_recording_pipeline`** unit test overlaps with golden replay; keep both (unit vs integration layout).

## Doc sync
- Root `README.md` milestone table already matched reality at bootstrap time.
- Crate READMEs reviewed; `velo-ffi` and `velo-rides` test sections updated.
- `STRAVA.md` present and referenced correctly.

## Test coverage added
- `crates/velo-ffi/tests/callback_round_trip.rs` — sensor samples update ride state; ERG/SIM trainer callbacks.
- `crates/velo-rides/tests/fit_artifacts_integration.rs` — encoded FIT bytes persist and re-parse from library paths.
