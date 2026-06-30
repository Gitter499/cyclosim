# velo-core

Simulation core — physics integrator, ride state machine, session recording, structured workouts, highlight clip planning, and FIT export orchestration.

## Key modules

| Module | Role |
|--------|------|
| `physics` | Rider model, `integrate_step`, `PhysicsConfig`, `RideSnapshot` |
| `ride` | `RideMode` (Free/ERG/SIM), live `RideState` |
| `ride_session` | Sample buffer, `RideSummary`, start/stop lifecycle |
| `workout` | `WorkoutEngine`, interval timeline, ERG target resolution (M5) |
| `zwo` | Zwift `.zwo` XML → `Workout` (M5) |
| `highlight` | `plan_highlight_clips` — post-ride moment windows (M5) |
| `route` / `packs` | `RouteModel`, route packs, scenery config |
| `app` | `VeloApp` — tick loop wiring sensors → physics → trainer |

## Dependencies

`velo-units`, `velo-platform`, `velo-fit`, `glam`, `thiserror`. No render, no SQLite, no UniFFI.

**Apple-symbol rule:** enforced by `scripts/lint-apple-symbols.sh` — zero Apple framework references.

## Test

```bash
cargo test -p velo-core          # unit + golden replay + route/workout/highlight integration
just lint                        # Apple-symbol check
```

Integration: `tests/golden_replay.rs`, `tests/route_ride.rs`, `tests/workout_erg.rs`, `tests/highlight_clips.rs`, `tests/zwo_import.rs`.

## Milestone

**M1** (physics) · **M2a** (ride loop) · **M2b** (session + FIT hookup) · **M3** (route + grade) · **M5 partial** (workout engine + highlight planning)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
