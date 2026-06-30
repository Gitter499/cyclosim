# Quality pass log

Running record of cross-cutting cleanup on the `dev` branch. See [.cursor/skills/quality-pass/SKILL.md](../.cursor/skills/quality-pass/SKILL.md) for the agent workflow.

| Date | Trigger | Summary | Commits |
|------|---------|---------|---------|
| 2026-06-30 | Initial bootstrap | Doc sync (plan M2c done, workspace layout); FFI callback + FIT catalog integration tests; test-helper warning cleanup | see below |

---

# Quality pass — 2026-06-30

## Scope
Milestone / trigger: **Initial quality pass bootstrap** on `dev` after M0–M2c monolith import.

## Changes made
- Marked **M2c** as done in `VeloSim-Technical-Plan.md`; clarified which crates exist in the workspace vs planned (M3+).
- Fixed `velo-core` crate description (removed stale "HUD state" — HUD lives in `velo-render`).
- Added FFI integration tests for sensor → `ride_state` and ERG/SIM trainer callback forwarding.
- Added cross-crate integration test: `velo-fit` encode → `velo-rides` artifact save → on-disk FIT parse.
- Silenced `dead_code` warnings in shared `velo-rides` test helpers used across integration test binaries.

## Findings (deferred)
- **Apple-symbol lint** only scans `velo-core`, `velo-units`, `velo-platform`; extend to `velo-fit` and `velo-rides` when convenient (they are portable today but unchecked).
- **`MockTrainerControl`** in `velo-platform` does not record commands; `velo-core` and FFI tests each use local recording stubs — unify in a future pass if test duplication grows.
- **`velo-core` `ride_recording_pipeline`** unit test overlaps with golden replay; keep both for now (unit vs integration file layout).
- **CI** runs on `main`/`master` only — consider adding `dev` branch when active development resumes.
- **Future crates** (`velo-route-import`, `velo-terrain`, `velo-cesium`, etc.) documented as planned; no scaffold until M3 agents land.

## Doc sync
- Root `README.md` milestone table already matched reality (M2c ✅, M3 next).
- Crate READMEs reviewed; `velo-ffi` and `velo-rides` test sections updated.
- `STRAVA.md` present and referenced correctly.

## Test coverage added
- `crates/velo-ffi/tests/callback_round_trip.rs` — sensor samples update ride state; ERG/SIM trainer callbacks.
- `crates/velo-rides/tests/fit_artifacts_integration.rs` — encoded FIT bytes persist and re-parse from library paths.

## Verification
- `cargo test` — all workspace tests pass (including 3 new integration tests).
- `./scripts/lint-apple-symbols.sh` — passed (`just` not installed in agent env; script run directly).
