# Quality pass log

Running record of cross-cutting cleanup on the `dev` branch. See [.cursor/skills/quality-pass/SKILL.md](../.cursor/skills/quality-pass/SKILL.md) for the agent workflow.

| Date | Trigger | Summary | Commits |
|------|---------|---------|---------|
| 2026-06-30 | Initial bootstrap | Doc sync (plan M2c done, workspace layout); FFI callback + FIT catalog integration tests; test-helper warning cleanup | `a1b2c3d`… (see git log before this pass) |
| 2026-06-30 | Post-M5 partial + multi-agent desync | Plan/README sync (M3b/M3c done, M5 partial); unified `RecordingTrainerControl`; doc refresh | see below |

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
- **FFI test stubs** — `RecordingTrainer` in `velo-ffi/tests/` still duplicates the platform helper because FFI uses `TrainerControlCallback`, not `TrainerControl`; unify only if a shared test crate is warranted.
- **Apple-symbol lint** still scans only `velo-core`, `velo-units`, `velo-platform`; extend to other portable crates when convenient.
- **M5 remaining** — in-app workout builder, highlight clips, full Liquid Glass setup/summary UI (#10).
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
- See post-M5 partial pass above for current deferred list; items below were addressed or superseded.
- **`velo-core` `ride_recording_pipeline`** unit test overlaps with golden replay; keep both (unit vs integration layout).

## Doc sync
- Root `README.md` milestone table already matched reality at bootstrap time.
- Crate READMEs reviewed; `velo-ffi` and `velo-rides` test sections updated.
- `STRAVA.md` present and referenced correctly.

## Test coverage added
- `crates/velo-ffi/tests/callback_round_trip.rs` — sensor samples update ride state; ERG/SIM trainer callbacks.
- `crates/velo-rides/tests/fit_artifacts_integration.rs` — encoded FIT bytes persist and re-parse from library paths.
