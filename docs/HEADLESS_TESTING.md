# Headless testing (no Kickr)

Automated coverage that mirrors **how a rider uses VeloSim** without a Wahoo Kickr, Apple Music authorization, or live BLE sensors. All flows use `velo-platform` trait mocks, FFI test doubles, or replay buffers.

## Quick start

```bash
# Rust — full workspace (primary gate)
cargo test --workspace

# Portable crate boundary
./scripts/lint-apple-symbols.sh

# Shell UI conventions
./scripts/lint-shell-ui.sh

# Swift — requires release FFI dylib
cargo build --release -p velo-ffi
cd shell-macos && swift build --product VeloSim
cd shell-macos && swift test   # needs DEVELOPER_DIR / Xcode for Metal bridge
```

## What simulates what

| Real hardware / service | Test substitute | Where |
|-------------------------|-----------------|-------|
| Wahoo Kickr (ERG/SIM) | `RecordingTrainerControl` / `RecordingTrainerCallback` | `velo-platform`, `velo-ffi/tests/common` |
| BLE power/cadence/HR | `MockSensorSource`, `TickSensors`, `ReplaySensors`, Swift `FakeSensorSource` | core + FFI + shell tests |
| Keyboard / AirPods steering | `MockSteeringInput`, `MockSteering`, `NoopSteeringInput` | core, FFI, shell |
| Apple Music segment playback | `RecordingAudioDirector`, `NoopAudioDirector` | core, FFI, shell |
| Strava upload | `MockPublisher` with fake activity URL | `velo-ffi/tests/common` |
| Screenshot / highlight encode | `MockMedia` (stub PNG + mock MP4 bytes) | `velo-ffi/tests/common` |
| GPX route file | `simple_climb.gpx` fixture | `velo-route-import/tests/fixtures/` |
| Zwift `.zwo` | Inline XML fixtures | `velo-core/tests/zwo_import.rs` |

## Test map → user story

| Test file | User story |
|-----------|------------|
| `velo-core/tests/scenarios/ride_lifecycle.rs` | Import climb → structured workout → 60 s replay → stop → FIT + highlight clips |
| `velo-core/tests/route_import_scenario.rs` | GPX import metadata drives route id, grade, ENU position |
| `velo-core/tests/steering_golden.rs` | Hold turn input → yaw integrates; recenter clears; deadzone suppresses drift |
| `velo-core/tests/audio_segment.rs` | Workout interval boundaries → segment energy + playback intent |
| `velo-core/tests/workout_erg.rs` | ERG targets step when intervals advance |
| `velo-core/tests/zwo_import.rs` | Import `.zwo` → validate → FTP-based engine targets |
| `velo-core/tests/highlight_clips.rs` | Post-ride moment windows (start, peak, mid, finish) |
| `velo-core/tests/golden_replay.rs` | Deterministic physics integrator golden distances |
| `velo-ffi/tests/app_scenarios.rs` | Full app: route + SIM ride + publish + library + FIT; workout ERG + audio; steering DTO |
| `velo-ffi/tests/ride_library_integration.rs` | Finish ride → SQLite row + highlight path |
| `velo-ffi/tests/workout_integration.rs` | Custom workout + `.zwo` parse via FFI |
| `velo-ffi/tests/steering_audio_integration.rs` | M6 steering + audio callbacks through handle |
| `velo-rides/tests/fit_artifacts_integration.rs` | FIT bytes → artifact paths → DB round-trip |
| `shell-macos/Tests/VeloSimTests/AppScenarioTests.swift` | Builder DTO mapping, steering modes, noop audio, FFI ride tick |
| `shell-macos/Tests/VeloSimTests/RideFlowTests.swift` | Start/stop ride, FIT export, local publish fallback |

## Pre hardware checklist

<!-- TODO: user assignment — manual Kickr / AirPods / MusicKit checklist before real-world testing -->

## CI notes

- GitHub Actions runs `cargo test --workspace` on PRs to `main`.
- Swift tests are **not** in CI by default (macOS 14 runner, Metal + prebuilt `libvelo_ffi` path). Run `swift test` locally before release.
- Liquid Glass (`VELO_LIQUID_GLASS`) is SDK-gated; headless tests do not exercise glass chrome.

See also [docs/QUALITY_PASS.md](QUALITY_PASS.md) and plan §19 in [VeloSim-Technical-Plan.md](../VeloSim-Technical-Plan.md).
