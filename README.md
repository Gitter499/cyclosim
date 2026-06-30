# VeloSim (cyclosim)

Native offline cycling simulator — Rust core + macOS Swift shell.

## M0 status

- Cargo workspace with `velo-units`, `velo-platform`, `velo-core`, `velo-render`, `velo-ffi`
- UniFFI round-trip: Swift toggles Rust state, fake `SensorSource` + `TrainerControl` callbacks
- wgpu Metal surface stub (solid-color clear)
- CI: `cargo test` + Apple symbol lint

## Build

```bash
# Rust only
cargo test

# Full app (requires Xcode Swift toolchain)
just build
just run
```

## Layout

See [VeloSim-Technical-Plan.md](VeloSim-Technical-Plan.md) for architecture and milestones.
