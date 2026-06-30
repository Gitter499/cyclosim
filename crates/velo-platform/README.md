# velo-platform

Platform abstraction — trait contracts between the Rust core and the Swift shell. No OS symbols, no I/O.

## Key modules

| Trait | Role |
|-------|------|
| `TrainerControl` | ERG/SIM commands to the smart trainer |
| `SensorSource` | Drain telemetry samples each tick |
| `AudioDirector`, `SteeringInput`, `Clock` | M6 — MusicKit segment playback + optional steering |
| `MockSensorSource`, `MockTrainerControl` | Headless tests (no-op trainer) |
| `RecordingTrainerControl` | Headless tests that assert ERG/SIM commands |

Types: `TelemetrySample`, `TrainerCaps`, `SegmentEnergy`, `PlaybackIntent`, `SteerState`.

## Dependencies

`velo-units` only. Shell implements traits over BLE, replay files, etc.; FFI uses UniFFI callbacks in `velo-ffi`.

**Apple-symbol rule:** CI lint fails if Apple framework names appear in this crate (or `velo-core` / `velo-units`).

## Test

```bash
cargo test -p velo-platform
```

## Milestone

**M0** (boundary spine) · exercised in **M2a** (real BLE)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
