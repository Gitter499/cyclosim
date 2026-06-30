# velo-rides

SQLite ride library — metadata catalog with on-disk artifact paths (FIT, PNG, highlight MP4). Replaces ad-hoc folder saves from M2b.

## Key types

| Item | Role |
|------|------|
| `RideLibrary` | Open DB, insert/list/get/delete rides |
| `RideRecord`, `NewRideRecord` | Row + insert DTO |
| `RideArtifacts` | Saved FIT/PNG paths under a ride UUID directory |
| `PublishStatus` | `Local` / `Strava` / `Failed` |
| `default_db_path`, `default_artifacts_base` | `~/Documents/VeloSim/` defaults |

Schema v2 in `schema.rs` (adds `highlight_clip_path`); migrations in `store.rs` / `schema::migrate`.

## Dependencies

`rusqlite` (bundled), `uuid`, `thiserror`. No UniFFI — exposed to Swift via `velo-ffi` (`RideLibraryHandle`).

**Apple-symbol rule:** portable storage layer; no Apple frameworks.

## Test

```bash
cargo test -p velo-rides    # schema, insert/query, v1→v2 migration, edge cases, FIT artifacts
```

## Milestone

**M2c** (ride history DB) · **M5** (`highlight_clip_path`, schema v2 migration)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
