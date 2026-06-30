# velo-rides

SQLite ride library — metadata catalog with on-disk artifact paths (FIT, PNG). Replaces ad-hoc folder saves from M2b.

## Key types

| Item | Role |
|------|------|
| `RideLibrary` | Open DB, insert/list/get/delete rides |
| `RideRecord`, `NewRideRecord` | Row + insert DTO |
| `RideArtifacts` | Saved FIT/PNG paths under a ride UUID directory |
| `PublishStatus` | `Local` / `Strava` / `Failed` |
| `default_db_path`, `default_artifacts_base` | `~/Documents/VeloSim/` defaults |

Schema in `schema.rs`; migrations in `store.rs`.

## Dependencies

`rusqlite` (bundled), `uuid`, `thiserror`. No UniFFI — exposed to Swift via `velo-ffi` (`RideLibraryHandle`).

**Apple-symbol rule:** portable storage layer; no Apple frameworks.

## Test

```bash
cargo test -p velo-rides    # schema, insert/query, migration, edge cases
```

## Milestone

**M2c** (ride history DB)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
