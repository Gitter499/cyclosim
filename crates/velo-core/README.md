# velo-core

Simulation core — physics integrator, ride state machine, session recording, and FIT export orchestration.

## Key modules

| Module | Role |
|--------|------|
| `physics` | Rider model, `integrate_step`, `PhysicsConfig`, `RideSnapshot` |
| `ride` | `RideMode` (Free/ERG/SIM), live `RideState` |
| `ride_session` | Sample buffer, `RideSummary`, start/stop lifecycle |
| `app` | `VeloApp` — tick loop wiring sensors → physics → trainer |

## Dependencies

`velo-units`, `velo-platform`, `velo-fit`, `glam`, `thiserror`. No render, no SQLite, no UniFFI.

**Apple-symbol rule:** enforced by `scripts/lint-apple-symbols.sh` — zero Apple framework references.

## Test

```bash
cargo test -p velo-core          # unit + golden replay
just lint                        # Apple-symbol check
```

Golden replay: `tests/golden_replay.rs` reproduces distance/time from recorded samples.

## Milestone

**M1** (physics) · **M2a** (ride loop) · **M2b** (session + FIT hookup)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
