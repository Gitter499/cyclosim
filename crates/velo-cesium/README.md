# velo-cesium

M3b glue for streaming 3D Tiles into VeloSim's wgpu renderer.

## Pinned dependency

| Component | Version |
|-----------|---------|
| **Cesium Native** | [`0.44.0`](https://github.com/CesiumGS/cesium-native/releases/tag/v0.44.0) |

API is pre-1.0 — bump the pin deliberately after validating upstream release notes.

## ToS guardrails (hard)

Tier B (Google Photorealistic 3D Tiles) is **online-only during the ride**:

- Tiles are fetched into **in-memory** buffers only — **never written to disk**.
- No offline tile cache, extraction, or derivation.
- Attribution string is surfaced to the HUD when 3D Tiles mode is active.

## Build (default — Rust glTF path)

```bash
cargo test -p velo-cesium
```

The default build decodes glTF/GLB in Rust and parses 3D Tiles tileset JSON. This is what CI runs.

## Optional: Cesium Native C++ bridge

Requires CMake, a C++17 compiler, and a checkout of Cesium Native:

```bash
git clone --branch v0.44.0 --depth 1 https://github.com/CesiumGS/cesium-native.git /path/to/cesium-native
export CESIUM_NATIVE_DIR=/path/to/cesium-native
cargo build -p velo-cesium --features cesium-native
```

Full CMake static-lib linking and the three Cesium integration interfaces
(`IAssetAccessor`, `ITaskProcessor`, `IPrepareRendererResources`) land in a follow-up PR
once the Rust-side mesh path is validated end-to-end.

## Dev tilesets

| Provider | Use |
|----------|-----|
| Cesium ion OSM Buildings | Default dev tileset (no Google creds required) |
| Google Photorealistic 3D Tiles | Production Tier B — requires Map Tiles API key |

Set `GOOGLE_MAP_TILES_API_KEY` for Google tiles. Without it, the session falls back to the
public Cesium ion sample asset for development.

The macOS shell stores keys in Keychain and injects them at runtime via `configure_runtime_secrets`
(see Settings). Env vars remain supported for CLI and tests.
