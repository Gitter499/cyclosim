# velo-core

Simulation core — physics integrator, ride state machine, session recording, structured workouts, and FIT export orchestration.

## Key modules

| Module | Role |
|--------|------|
| `physics` | Rider model, `integrate_step`, `PhysicsConfig`, `RideSnapshot` |
| `ride` | `RideMode` (Free/ERG/SIM), live `RideState` |
| `ride_session` | Sample buffer, `RideSummary`, start/stop lifecycle |
| `workout` | `WorkoutEngine`, interval timeline, ERG target resolution (M5) |
| `route` / `packs` | `RouteModel`, route packs, scenery config |
| `app` | `VeloApp` — tick loop wiring sensors → physics → trainer |

## Dependencies

`velo-units`, `velo-platform`, `velo-fit`, `glam`, `thiserror`. No render, no SQLite, no UniFFI.

**Apple-symbol rule:** enforced by `scripts/lint-apple-symbols.sh` — zero Apple framework references.

## Test

```bash
cargo test -p velo-core          # unit + golden replay + route/workout integration
just lint                        # Apple-symbol check
```

Integration: `tests/golden_replay.rs`, `tests/route_ride.rs`, `tests/workout_erg.rs`.

## Milestone

**M1** (physics) · **M2a** (ride loop) · **M2b** (session + FIT hookup) · **M3** (route + grade) · **M5 partial** (workout engine)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
