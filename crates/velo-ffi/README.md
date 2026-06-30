# velo-ffi

UniFFI surface — exposes Rust core, renderer, and ride library to the macOS Swift shell.

## Key exports

| Item | Role |
|------|------|
| `VeloHandle` | App lifecycle: tick, ride modes, renderer, publish flow |
| `RideLibraryHandle` | Standalone DB access (list/get/delete) |
| Callback traits | `SensorSourceCallback`, `TrainerControlCallback`, `MediaCaptureCallback`, `ActivityPublisherCallback` |
| DTOs | `RideStateDto`, `RideSummaryDto`, `FramebufferDto`, `RideRecordDto`, … |

Builds as `lib`, `staticlib`, and `cdylib` (`velo_ffi`). Swift bindings generated via `just bindgen`.

## Dependencies

Wires `velo-core`, `velo-platform`, `velo-render`, `velo-rides`, `velo-units`, `uniffi`. This is the **only** crate the shell links directly.

Shell owns Apple-only work: PNG encode (VideoToolbox), Strava OAuth, CoreBluetooth FTMS.

## Test

```bash
cargo test -p velo-ffi
cargo build --release -p velo-ffi
just bindgen    # regenerate Swift stubs
```

Integration: `tests/callback_round_trip.rs`, `tests/ride_library_integration.rs`.

## Milestone

**M0** (round-trip) · **M2a–M2c** (full ride + publish + library FFI)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
