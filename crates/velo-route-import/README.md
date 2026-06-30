# velo-route-import

Parse activity files (GPX required; TCX basic; FIT stubbed) into a resampled [`RouteModel`](../velo-core/) for route packs.

## Pipeline

1. Parse track points (lat/lon/ele)
2. Resample to ~8 m spacing (configurable)
3. Smooth/fill elevation
4. Compute grade with a 50 m smoothing window
5. Write `route.json` + `meta.json` into a route pack directory

## CLI

```bash
cargo run -p velo-route-import -- \
  -i ride.gpx \
  -o ~/Documents/VeloSim/packs/my-route \
  --route-id my-route \
  --name "My Route"
```

## Library

```rust
use velo_route_import::import_gpx;

let model = import_gpx(&gpx_bytes, "id", "Name", 8.0, 50.0)?;
model.save_pack(pack_dir)?;
```

## Tests

Fixture GPX in `tests/fixtures/`. CI runs fully offline.

```bash
cargo test -p velo-route-import
```
