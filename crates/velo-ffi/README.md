# velo-ffi

UniFFI surface — exposes Rust core, renderer, and ride library to the macOS Swift shell.

## Key exports

| Item | Role |
|------|------|
| `VeloHandle` | App lifecycle: tick, ride modes, renderer, publish flow, workouts, bikes, scenery |
| `RideLibraryHandle` | Standalone DB access (list/get/delete) |
| Callback traits | `SensorSourceCallback`, `TrainerControlCallback`, `MediaCaptureCallback`, `ActivityPublisherCallback` |
| DTOs | `RideStateDto`, `RideSummaryDto`, `FramebufferDto`, `RideRecordDto`, `WorkoutLiveDto`, … |

Builds as `lib`, `staticlib`, and `cdylib` (`velo_ffi`). Swift bindings generated via `just bindgen`.

## Dependencies

Wires `velo-core`, `velo-platform`, `velo-render`, `velo-rides`, `velo-route-import`, `velo-terrain`, `velo-cesium`, `velo-bikegen`, `velo-units`, `uniffi`. This is the **only** crate the shell links directly.

Shell owns Apple-only work: PNG encode (VideoToolbox), Strava OAuth, CoreBluetooth FTMS.

## Test

```bash
cargo test -p velo-ffi
cargo build --release -p velo-ffi
just bindgen    # regenerate Swift stubs
```

Integration tests under `tests/`:

| File | Coverage |
|------|----------|
| `callback_round_trip.rs` | Sensor → ride state; ERG/SIM trainer callbacks |
| `ride_library_integration.rs` | Publish flow + SQLite catalog |
| `route_import_integration.rs` | GPX import → route pack FFI |
| `tiles_integration.rs` | Scenery config + synthetic 3D Tiles session |
| `bike_integration.rs` | Bike import, list, active bike |
| `workout_integration.rs` | Sample workout start, live state, ERG target |

## Milestone

**M0** (round-trip) · **M2a–M2c** (full ride + publish + library FFI) · **M3–M3c** (route, tiles, bikes) · **M5 partial** (workout FFI)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
