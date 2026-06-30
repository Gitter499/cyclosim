# velo-terrain

Offline terrain bake for VeloSim route packs: heightfield → triangulated mesh + texture atlas.

## Route pack terrain files

Written alongside `route.json` from `velo-route-import`:

| File | Description |
|------|-------------|
| `terrain.mesh.bin` | Binary mesh (`VTM1` magic): positions (f32×3) + UVs (f32×2) + u32 indices |
| `terrain.png` | RGBA8 satellite-style texture (procedural fallback in CI/offline dev) |
| `terrain.json` | Vertex/index counts + texture dimensions |

Default corridor: **200 m** width; cell size **10 m** (see plan §0).

## Pipeline

1. Load `route.json` from pack
2. Build heightfield (synthetic DEM for offline/CI; Copernicus GLO-30 behind optional `network-fetch` feature — not required for M3)
3. Triangulate mesh in local ENU (origin = route start)
4. Generate procedural earth-tone texture (or raster tiles when network bake enabled)
5. Write pack files

## CLI

```bash
# After route import:
cargo run -p velo-terrain -- --pack ~/Documents/VeloSim/packs/my-route
```

## Library

```rust
velo_terrain::bake_terrain_pack(pack_dir, 200.0, 10.0)?;
let pack = velo_terrain::TerrainPack::load_from_dir(pack_dir)?;
```

## Tests

Synthetic DEM only — no network in CI.

```bash
cargo test -p velo-terrain
```
