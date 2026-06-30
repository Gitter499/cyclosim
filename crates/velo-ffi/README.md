# velo-ffi

UniFFI surface — exposes Rust core, renderer, and ride library to the macOS Swift shell.

## Key exports

| Item | Role |
|------|------|
| `VeloHandle` | App lifecycle: tick, ride modes, renderer, publish flow, workouts, bikes, scenery |
| `RideLibraryHandle` | Standalone DB access (list/get/delete) |
| Callback traits | `SensorSourceCallback`, `TrainerControlCallback`, `MediaCaptureCallback`, `ActivityPublisherCallback`, `SteeringInputCallback`, `AudioDirectorCallback` |
| DTOs | `RideStateDto`, `RideSummaryDto`, `HighlightClipRequestDto`, `FramebufferDto`, `RideRecordDto`, `WorkoutDto`, `WorkoutLiveDto`, … |
| Functions | `parse_zwo_xml` — Zwift `.zwo` → `WorkoutDto` |

`MediaCaptureCallback` — shell implements PNG encode (`encode_png_rgba`) and highlight reel encode (`encode_highlight_clip` from ring-buffer frames).

`SteeringInputCallback` — shell polls keyboard or AirPods head motion each tick; core applies deadzone, smoothing, and camera yaw.

`AudioDirectorCallback` — core emits `on_segment(energy, intent)` at workout interval boundaries; shell maps to Apple Music playback (control only, no raw PCM).

Register audio via `VeloHandle::set_audio_director`; pass steering to `tick` alongside sensors and trainer.

Builds as `lib`, `staticlib`, and `cdylib` (`velo_ffi`). Swift bindings generated via `just bindgen`.

## Dependencies

Wires `velo-core`, `velo-platform`, `velo-render`, `velo-rides`, `velo-route-import`, `velo-terrain`, `velo-cesium`, `velo-bikegen`, `velo-units`, `uniffi`. This is the **only** crate the shell links directly.

Shell owns Apple-only work: PNG/H.264 encode (VideoToolbox), Strava OAuth, CoreBluetooth FTMS.

## Test

```bash
cargo test -p velo-ffi
cargo build --release -p velo-ffi
just bindgen    # regenerate Swift stubs
```

Integration tests under `tests/` (shared mocks in `tests/common/mod.rs`):

| File | Coverage |
|------|----------|
| `callback_round_trip.rs` | Sensor → ride state; ERG/SIM trainer callbacks |
| `ride_library_integration.rs` | Publish flow, SQLite catalog, highlight clip encode + persist |
| `route_import_integration.rs` | GPX import → route pack FFI |
| `tiles_integration.rs` | Scenery config + synthetic 3D Tiles session |
| `bike_integration.rs` | Bike import, list, active bike |
| `workout_integration.rs` | Custom workout start, validation, live state, ERG target, `.zwo` parse |
| `steering_audio_integration.rs` | Steering axis → ride state; workout → audio director |
| `app_scenarios.rs` | End-to-end route + publish + workout ERG + steering (no hardware) |

## Milestone

**M0** (round-trip) · **M2a–M2c** (full ride + publish + library FFI) · **M3–M3c** (route, tiles, bikes) · **M5** (workout builder FFI, `.zwo` parse, highlight clip encode) · **M6** (`SteeringInputCallback`, `AudioDirectorCallback`)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
