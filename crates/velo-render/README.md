# velo-render

wgpu renderer — terrain mesh (route pack) or flat ground plane, chase camera along route ENU, glyphon HUD overlay.

## Key types

| Item | Role |
|------|------|
| `Renderer` | Metal surface via `from_metal_layer` (macOS); resize + `render_frame` |
| `HudRenderer`, `HudSnapshot` | Power, cadence, HR, speed, distance, grade, mode |
| `ChaseCamera`, `GroundMesh`, `TerrainScene` | Scene pass (flat fallback or textured terrain) |
| `RouteFollow` | Chase camera position along loaded route |
| `capture_framebuffer_rgba`, `FramebufferRgba` | RGBA8 readback for M2b PNG |
| `headless_ok()` | CI placeholder on non-macOS hosts |

## Dependencies

`wgpu`, `glyphon`, `glam`, `bytemuck`. Metal layer binding is `#[cfg(target_os = "macos")]` only — no AppKit in Rust.

Consumed by `velo-ffi`; not linked from `velo-core`.

## Test

```bash
cargo test -p velo-render    # unit tests; GPU init needs macOS + Metal layer
```

## Milestone

**M0** (surface stub) · **M2a** (HUD + ground plane) · **M2b** (screenshot capture) · **M3** (terrain mesh from route pack)

Architecture: [VeloSim-Technical-Plan.md](../../VeloSim-Technical-Plan.md)
