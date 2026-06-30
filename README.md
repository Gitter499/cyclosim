# VeloSim (cyclosim)

Native offline cycling simulator — portable Rust core + thin macOS Swift shell.

## Milestone status

| Milestone | Status |
|-----------|--------|
| **M0** — Skeleton & boundary | ✅ |
| **M1** — Physics core | ✅ |
| **M2a** — Trainer + HUD | ✅ |
| **M2b** — FIT + Strava + screenshot | ✅ |
| **M2c** — Ride library (SQLite) | ✅ |
| **M3** — Real route + terrain | ✅ |
| **M3b** — Google 3D Tiles (spike) | ✅ |

See [VeloSim-Technical-Plan.md](VeloSim-Technical-Plan.md) for acceptance criteria and architecture.

## Crates

| Crate | Purpose |
|-------|---------|
| [`velo-units`](crates/velo-units/) | Physical quantity newtypes |
| [`velo-platform`](crates/velo-platform/) | Shell↔core trait contracts |
| [`velo-core`](crates/velo-core/) | Physics, ride loop, session |
| [`velo-render`](crates/velo-render/) | wgpu scene + HUD |
| [`velo-fit`](crates/velo-fit/) | FIT activity encoder |
| [`velo-rides`](crates/velo-rides/) | SQLite ride library |
| [`velo-route-import`](crates/velo-route-import/) | GPX/TCX → RouteModel (lib + CLI) |
| [`velo-terrain`](crates/velo-terrain/) | DEM → terrain mesh + texture bake |
| [`velo-cesium`](crates/velo-cesium/) | 3D Tiles streaming + glTF decode (M3b) |
| [`velo-ffi`](crates/velo-ffi/) | UniFFI surface for Swift |
| [`shell-macos`](shell-macos/) | macOS app (BLE, Strava, UI) |

Portable crates (`velo-units`, `velo-platform`, `velo-core`) must not reference Apple frameworks — enforced by `just lint`.

## Build

```bash
cargo test              # Rust workspace
just lint               # Apple-symbol check
just build && just run  # Full app (Xcode Swift + Metal)
```

### M3b build notes (optional Cesium Native)

Default build uses Rust glTF decode (CI-friendly). See [`crates/velo-cesium/README.md`](crates/velo-cesium/README.md) for pinned Cesium Native **0.44.0** and optional `cesium-native` feature.

## Git history

- **`main`** — stable milestone snapshots and documentation.
- **`dev`** — active development; quality passes land here before merging to `main`.
- **`initial-monolith`** tag — single commit with the full M0–M2c codebase (first import).
- Following commits document each crate/feature (`docs(...)` messages per milestone).
- Post-milestone cleanup: [.cursor/skills/quality-pass/SKILL.md](.cursor/skills/quality-pass/SKILL.md) · [docs/QUALITY_PASS.md](docs/QUALITY_PASS.md)
- Strava setup: [STRAVA.md](STRAVA.md).
