# VeloSim (cyclosim)

Native offline cycling simulator. Portable Rust core, thin macOS Swift shell.

## Milestone status

| Milestone | Status | Issue |
|-----------|--------|-------|
| **M0** — Skeleton & boundary | ✅ | [#1](https://github.com/Gitter499/cyclosim/issues/1) |
| **M1** — Physics core | ✅ | [#2](https://github.com/Gitter499/cyclosim/issues/2) |
| **M2a** — Trainer + HUD | ✅ | [#3](https://github.com/Gitter499/cyclosim/issues/3) |
| **M2b** — FIT + Strava + screenshot | ✅ | [#4](https://github.com/Gitter499/cyclosim/issues/4) |
| **M2c** — Ride library (SQLite) | ✅ | [#5](https://github.com/Gitter499/cyclosim/issues/5) |
| **M3** — Real route + terrain | ✅ | [#6](https://github.com/Gitter499/cyclosim/issues/6) |
| **M3b** — Google 3D Tiles | 🔜 | [#7](https://github.com/Gitter499/cyclosim/issues/7) |

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
| [`velo-ffi`](crates/velo-ffi/) | UniFFI surface for Swift |
| [`shell-macos`](shell-macos/) | macOS app (BLE, Strava, UI) |

Portable crates (`velo-units`, `velo-platform`, `velo-core`) must not reference Apple frameworks. Enforced by `just lint`.

## Build

```bash
cargo test              # Rust workspace
just lint               # Apple-symbol check
just build && just run  # Full app (Xcode Swift + Metal)
```

## Git workflow

All work uses [GitHub issues](https://github.com/Gitter499/cyclosim/issues) and pull requests.

| Branch | Role |
|--------|------|
| **`main`** | Stable milestone snapshots |
| **`dev`** | Integration; open PRs here |
| Feature branches | `feat/issue-N-name` from `dev` |

- Reference issues in PR titles and use `Closes #N` in PR bodies.
- Granular commits per feature slice.
- Post-milestone cleanup: [.cursor/skills/quality-pass/SKILL.md](.cursor/skills/quality-pass/SKILL.md) · [docs/QUALITY_PASS.md](docs/QUALITY_PASS.md)
- Strava setup: [STRAVA.md](STRAVA.md)
- **`initial-monolith`** tag: single import commit (M0–M2c baseline)

## Agent rules

- Read this file and the technical plan before implementing a milestone.
- One issue per PR; run `cargo test` and `just lint` before pushing.
- Do not rewrite git history on `main`.
- **No AI tool branding** in commits, PRs, or user-facing docs. The user handles attribution.
