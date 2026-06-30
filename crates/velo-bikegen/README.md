# velo-bikegen

Offline bike asset pipeline for VeloSim **M3c**: import 1–4 photos → normalized glTF in the bike library.

## Library layout

```
~/Documents/VeloSim/bikes/<bike-id>/
├── bike.json          # metadata + anchor transform
├── bike.glb           # normalized glTF binary
└── sources/           # copied input images
```

## v1 behavior (offline / CI)

Without a GPU or hosted image-to-3D API, `velo-bikegen` writes a **synthetic placeholder glTF** tinted from source image colors and scaled to a standard wheelbase (~1.05 m). This keeps CI and personal dev fully offline.

Real image-to-3D (TRELLIS.2, Hunyuan3D, Meshy, Tripo) is deferred; enable the `hosted-api` feature for the scaffold only.

## CLI

```bash
cargo build -p velo-bikegen --bin velo-bikegen

# Import photos
velo-bikegen import photo1.png photo2.png --id my-bike --name "My Bike"

# List bikes
velo-bikegen list

# Show asset paths
velo-bikegen show my-bike
```

## Rust API

```rust
use velo_bikegen::{default_bikes_dir, import_bike_from_images, list_bikes, load_bike_asset};

let dir = default_bikes_dir();
let asset = import_bike_from_images(&dir, &[path1, path2], "my-bike", Some("My Bike"))?;
// asset.gltf_path, asset.anchor → velo-render foreground pass
```

## Integration

- **velo-render**: `Renderer::load_bike_gltf(path, anchor)` draws the model in the foreground-object pass (chase camera).
- **velo-ffi / shell**: `import_bike_from_images`, `list_bikes`, `set_active_bike` — Swift file picker + bike picker in the ride UI.

## Deferred

- Local TRELLIS.2 / Hunyuan3D inference (16–24 GB VRAM)
- Hosted Meshy/Tripo API wiring (`hosted-api` feature scaffold)
- PBR textures and mesh cleanup for thin structures (spokes, cables)
