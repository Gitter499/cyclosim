# VeloSim — Technical Implementation Plan

> Working title. A native, offline ("solo") cycling simulator that renders real-world
> routes in 3D, driven by a smart trainer (Kickr) and sensors, with optional
> Gaussian-splat scenery, segment-aware Apple Music, and AirPods head-tracked steering.
> Primary target: macOS on Apple Silicon (M4 Air). Designed so the core is portable and
> only the shell is Apple-specific.

---

## 0. Decisions locked

| Decision | Choice | Consequence |
|---|---|---|
| Stack boundary | **Rust sim core + thin Swift shell** | Core imports zero Apple symbols; shell is replaceable per-platform |
| Distribution | Personal use, **not Apple-locked** | Portability is a hard architectural constraint, not a nice-to-have |
| Scenery (fidelity ceiling) | **Offline-baked Gaussian splats per segment** | Heavy GPU bake; deferred (Tier C), not the starting point |
| Scene-gen **fast path** | **Mesh first, then Google 3D Tiles direct-render** (no GPU bake) | Photorealistic scenery, zero preprocessing; 3D Tiles is **online-only, render-only** (ToS: no caching/extraction) |
| Physics impl | **Hand-rolled fixed-step integrator + `glam`** | Not a physics engine (Rapier is wrong category); deterministic; ~15 lines |
| World renderer | **Cesium Native (C++) via `cxx` + own wgpu/Metal draw** | Cesium Native does streaming/LOD/glTF decode (no rendering); we draw. Native perf; renderer stays cross-platform |
| Bike models | **Image-to-3D import (TRELLIS.2 / Hunyuan3D or hosted API) → glTF** | Rider's bike from 2D web photos; foreground object; offline asset-gen, trivial render integration |
| v1 scope | **Full parity**, priority order: trainer+physics → scenery → music/AirPods | Phase ordering follows this exactly |
| Scenery refinement model | **FLUX.1-schnell** (SDEdit partial-denoise) | Few-step distilled, Apache-2.0 (redistributable); general model, no task-specific fine-tune |
| Splat compression | **SOG/SPZ** (proven, ~10–20×); HAC++ as a *future* option | Hard ≤40 GB total budget; hero-only splats + segment dedup + LOD |
| Strava + media | **Lands at M2b** (FIT upload + auto screenshots) | Decoupled from scenery; ships right after the first rideable build (M2a) |
| Repo / workspace name | **`cyclosim`** (matches git repo) | Crate names stay `velo-*`; only the top-level directory uses the repo name |
| Tier A terrain defaults | **Copernicus GLO-30 + free/low-cost raster**; 200 m corridor (50 m near route) | Locked before M3; avoids Mapbox dependency until needed |
| Tier B routing | **Auto-detect Google coverage at import**; Tier A when unknown; **opt-in per route** if quota/cost is a concern | Tier A is always the default fallback; Tier B is the photoreal upgrade |
| Workouts (v1) | **In-app builder only**; `.zwo` import is a fast follow | Unblocks M4 without import-format work |
| Hero segments (Tier C) | **Manual marking only for v1** | Auto-by-gradient is post-v1 polish |
| Cesium Native version | **Pin a specific release** (API pre-1.0) | Validated in M3b-spike before full M3b integration |

**Sequencing note (read this):** the splat bake is the highest-risk component and is *not*
on the critical path to the first rideable build. The renderer is built against a textured
terrain mesh first; splats are composited on top once the substrate, ride loop, and trainer
control are proven. Mesh is permanent (it's the terrain), splats are an overlay for hero detail.

---

## 1. Guiding principles

1. **Core is pure and portable.** The `velo-core` crate has no dependency on CoreBluetooth,
   MusicKit, AppKit, Metal, or anything Apple. It talks to the world exclusively through
   traits defined in `velo-platform` (§4). A future Linux/Windows shell implements the same
   traits over BlueZ / WinRT / etc.
2. **Determinism.** The simulation is a fixed-timestep, deterministic state machine. Given a
   recorded telemetry stream + route, replaying produces bit-identical state. This makes the
   physics testable and makes "ghost"/replay rides trivial later.
3. **Render backend is already portable.** Use **wgpu** (targets Metal on Apple Silicon, Vulkan/DX
   elsewhere). The core emits a render-agnostic scene description; the renderer is in Rust, not Swift.
   Swift owns *only* the OS chrome it must (window, Liquid Glass UI, the three Apple-only APIs).
4. **Capabilities cross trait boundaries, never concrete types.** Trainer, sensors, audio, input,
   storage, and the route-source importer are all traits. The shell injects implementations at startup.
5. **Offline-first (with one online exception).** No multiplayer, no server. The custom bake path and
   terrain mesh are fully offline. The Google 3D Tiles fast path (Tier B) is the **one exception**: per
   Google's ToS its tiles **cannot be cached or used offline**, so 3D Tiles routes stream live during the
   ride. Want a fully offline ride? Use Tier A/C, not Tier B.

---

## 2. System architecture

```
        ┌──────────────────────── Swift Platform Shell (Apple-only) ────────────────────────┐
        │  Liquid Glass UI (SwiftUI)   CoreBluetooth   MusicKit   CMHeadphoneMotionManager   │
        │  Window/lifecycle (AppKit)   File pickers                                          │
        └───────────────▲───────────────────────────────────────────────▲───────────────────┘
                        │  implements velo-platform traits via FFI (UniFFI)│  emits events / reads state
        ┌───────────────┴───────────────────────────────────────────────┴───────────────────┐
        │                                  velo-platform (traits only)                         │
        │   TrainerControl   SensorSource   AudioDirector   SteeringInput   Storage   Clock     │
        └───────────────▲───────────────────────────────────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────────────────────────────────────────────────────────┐
        │                                      velo-core (Rust)                                │
        │  ride loop (fixed dt)  physics/power  route model  workout engine  HUD state         │
        │  scene graph (render-agnostic)                                                       │
        └───────────────▲───────────────────────────────────────────────────────────────────┘
                        │ scene description
        ┌───────────────┴────────────┐     ┌──────────────────────────────────────────────────┐
        │  velo-render (wgpu/Rust)    │     │  Offline tools (separate binaries, run pre-ride)  │
        │  terrain mesh + splat passes│     │  velo-route-import   velo-terrain   VeloSplatBake  │
        │  + HUD overlay              │     └──────────────────────────────────────────────────┘
        └─────────────────────────────┘
```

---

## 3. Repository & crate layout

Cargo workspace. The Xcode project links the produced static lib + generated UniFFI bindings.

```
cyclosim/                           # repo root (git: cyclosim; product: VeloSim)
├── Cargo.toml                      # workspace
├── crates/
│   ├── velo-core/                  # sim, physics, route model, workout engine, HUD state, scene graph
│   ├── velo-platform/              # TRAITS ONLY: TrainerControl, SensorSource, AudioDirector, …
│   ├── velo-render/                # wgpu renderer: terrain pass, splat pass, foreground-object pass, HUD overlay
│   ├── velo-cesium/                # `cxx` bridge to Cesium Native (C++): 3D Tiles streaming/LOD/glTF decode
│   ├── velo-units/                 # newtype units (Watts, Meters, Grade, MetersPerSecond, …)
│   ├── velo-route-import/          # GPX/TCX/FIT/RWGPS → RouteModel (lib + thin CLI)
│   ├── velo-rides/                 # ride library: SQLite catalog + on-disk artifact paths (FIT, PNG, clips)
│   ├── velo-terrain/               # raster tiles + Terrain-RGB → TerrainTile mesh (lib + CLI)
│   ├── velo-splatbake/             # "VeloSplatBake" offline GS bake (CLI) — heaviest, slowest
│   ├── velo-bikegen/               # bike import: 2D images → image-to-3D → glTF asset (lib + CLI)
│   └── velo-ffi/                   # UniFFI surface: exposes core+render to Swift
├── shell-macos/                    # Xcode project (Swift): UI, CoreBluetooth, MusicKit, AirPods
│   └── …
└── assets/
    └── packs/                      # baked route packs: mesh + splats + route + metadata
```

**Rule enforced in CI:** `velo-core`, `velo-units`, `velo-platform` have an empty allow-list of
OS frameworks. A grep/lint step fails the build if any Apple symbol leaks below the shell.

---

## 4. Platform abstraction layer (`velo-platform`) — the portability spine

These traits are the contract between core and shell. Get these signatures right early; they
rarely change and everything else hangs off them. Sketch (Rust, async via `async-trait` or channels):

```rust
/// Commands the app sends TO the trainer.
pub trait TrainerControl: Send + Sync {
    /// ERG mode: hold this exact power regardless of cadence.
    fn set_target_power(&self, watts: Watts);
    /// SIM mode: trainer sets resistance from physical params; rider produces power.
    fn set_simulation(&self, grade: Grade, crr: f32, cw_a: f32);   // FTMS sim params
    fn stop(&self);
    fn capabilities(&self) -> TrainerCaps;                          // ERG? SIM? max watts? grade range?
}

/// Telemetry coming FROM trainer + sensors (push stream).
pub trait SensorSource: Send + Sync {
    /// Hot stream of samples; core selects/fuses by type.
    fn subscribe(&self) -> Receiver<TelemetrySample>;
}
pub struct TelemetrySample {
    pub at: Instant,
    pub power: Option<Watts>,
    pub cadence: Option<Rpm>,
    pub heart_rate: Option<Bpm>,
    pub wheel_speed: Option<MetersPerSecond>, // if a speed sensor present
}

/// Audio is a DIRECTOR, not a mixer (see §13 for why).
pub trait AudioDirector: Send + Sync {
    fn on_segment(&self, energy: SegmentEnergy, intent: PlaybackIntent); // shell maps to MusicKit etc.
}

/// Steering axis in [-1.0, 1.0]; shell maps AirPods/keyboard/gamepad to it.
pub trait SteeringInput: Send + Sync {
    fn poll(&self) -> SteerState;     // axis + recenter request
}

pub trait Storage: Send + Sync { /* read/write ride logs, settings, route packs */ }
pub trait Clock: Send + Sync { fn now(&self) -> Instant; } // injectable for deterministic replay

/// Frame/clip capture. Core asks for a grab at a moment; shell does the platform encode.
pub trait MediaCapture: Send + Sync {
    fn grab_screenshot(&self) -> ImageHandle;                 // framebuffer → PNG/JPEG
    fn record_clip(&self, frames: FrameRange) -> ClipHandle;  // replay-camera frames → H.264/HEVC
}

/// Publish a finished ride. Core writes the FIT; shell handles OAuth + upload (or a Rust HTTP impl).
pub trait ActivityPublisher: Send + Sync {
    fn upload(&self, fit: FitFile, media: Vec<MediaHandle>) -> Result<ActivityUrl>;
}
```

The Swift shell implements `TrainerControl`/`SensorSource` over CoreBluetooth, `AudioDirector`
over MusicKit, `SteeringInput` over `CMHeadphoneMotionManager`, `MediaCapture` over VideoToolbox,
and `ActivityPublisher` over the Strava API. A Linux shell would implement the
same traits over BlueZ + MPRIS + a gamepad. **The core cannot tell the difference.**

---

## 5. Domain model & units

- **`velo-units`**: newtypes (`Watts`, `Meters`, `MetersPerSecond`, `Grade`, `Kilograms`, `Rpm`, `Bpm`).
  No bare `f32` for physical quantities in core APIs — kills an entire class of unit bugs.
- **`RouteModel`**: ordered list of `RoutePoint { distance_m, lat, lon, elevation_m, grade }` with
  derived cumulative distance and smoothed grade. Loaded from a route pack (§8).
- **`Rider`**: mass, bike mass, `Crr`, `CdA`, drivetrain efficiency, FTP (for workout %).
- **`Workout`**: timeline of `Interval { duration_or_distance, target }` where target is
  ERG watts, %FTP, or "free ride" (SIM). Drives ERG commands and HUD targets.
- **`RideState`**: position along route, speed, distance, elapsed, current grade, current
  telemetry, current workout interval. This is the single source of truth the HUD and renderer read.

---

## 6. Physics & power simulation  *(Phase priority #1)*

The trainer reports the rider's **power**; the sim integrates that into **speed and position**
along the route. This is the inverse of the classic cycling power equation.

**Resistive forces** at speed `v` on grade `θ` (θ = atan(grade)):

```
F_gravity  = m · g · sin(θ)
F_rolling  = m · g · Crr · cos(θ)
F_drag     = 0.5 · ρ · CdA · (v + v_wind)²        # v_wind ≈ 0 indoors unless modeled
F_resist   = F_gravity + F_rolling + F_drag
```

**Forward integration** (fixed dt, e.g. 100 Hz internal tick, decoupled from render fps):

```
P_wheel  = P_rider · drivetrain_efficiency
F_propel = P_wheel / max(v, v_min)                # guard v→0 to avoid divide blowup
a        = (F_propel − F_resist) / m_effective    # m_effective folds in wheel rotational inertia
v        = max(0, v + a · dt)
distance += v · dt
```

- `m_effective = m + I_wheels / r²` (rotational inertia). Small but matters for accel feel.
- At `v ≈ 0`, switch to a startup model so the rider can accelerate from a stop without a singularity.
- **ERG mode**: app commands `set_target_power(target)`; the trainer holds it; the rider's *actual*
  power (from telemetry) is what you integrate. Cadence affects feel, not the held wattage.
- **SIM mode**: app sends grade via `set_simulation`; trainer sets resistance; rider's produced
  power drives the integration above. This is the "free ride the real route" mode.

**Implementation note — no physics engine.** This is 1-D longitudinal dynamics (a closed-form force
equation + a fixed-step integrator), not rigid-body/collision physics. **Do not use Rapier or any physics
engine** — it's the wrong category (built for contacts, joints, ragdolls) and its adaptive/solver
machinery fights the bit-exact replay requirement. Hand-roll the fixed-step integrator (~15 lines); use
`glam` for vector/camera math (don't hand-roll linear algebra). The genuinely hard part isn't the
integration — it's empirical tuning of `CdA`, `Crr`, drivetrain efficiency, trainer power smoothing, and
the `v→0` startup, calibrated against real `.fit` files. (Rapier would only become relevant if steering
ever grows into real lean/cornering dynamics — and even then a small custom 2-D model likely beats it.)

**Determinism & testing:** the integrator is a pure function `(RideState, TelemetrySample, dt) -> RideState`.
Unit-test against hand-computed cases (flat steady state, known climb, coast-down). Golden-file test:
replay a recorded `.fit` over a known route and assert final distance/time within tolerance of real ride.

---

## 7. Trainer & sensor control  *(Phase priority #1, with §6)*

**Protocol (BLE, no ANT+ on Mac without a dongle):**
- **FTMS (Fitness Machine Service)** is the modern standard the Kickr speaks:
  - *Indoor Bike Data* characteristic → notifications with instantaneous power, cadence, speed.
  - *Fitness Machine Control Point* → write `Set Target Power` (ERG) or
    `Set Indoor Bike Simulation Parameters` (grade, wind speed, Crr, Cw) (SIM).
  - *Fitness Machine Feature* / *Supported Power Range* → capabilities, max watts, grade range.
- **Heart Rate Service** (standard) for HR straps. **CSC** (Cycling Speed & Cadence) for separate sensors.
- Note Wahoo also exposes legacy proprietary characteristics; **target FTMS first**, fall back only if needed.
- Keep FE-C (ANT+) as a *documented future shell capability* — same `TrainerControl` trait, different impl.

**Shell responsibilities (Swift / CoreBluetooth):** scan → filter by advertised services →
pair → subscribe to notifications → expose as `SensorSource` stream → translate `TrainerControl`
calls into Control Point writes. Robust reconnect/state-restoration (`CBCentralManager` restoration)
is part of "feels like Zwift." Pairing UX lives in the Liquid Glass shell, but the *device model*
(what's connected, what role) lives in core state so the HUD can read it.

**Dev without hardware:** a **recorded FTMS trace replay** mode (Swift or Rust mock `SensorSource`
feeding a captured notification log) lets CI and headless dev proceed with the UX every session.
Real-hardware validation remains a manual M2a checklist.

**Core responsibilities:** sensor fusion (pick the best power source if multiple), dropout handling
(hold last value briefly, then flag stale), and mapping workout intervals → `set_target_power` /
`set_simulation` commands at interval boundaries.

---

## 8. Route ingestion (`velo-route-import`)

- Input formats: **GPX, TCX, FIT**, and RideWithGPS exports (which are GPX/FIT under the hood).
  RWGPS also has an API, but file import covers personal use without API-key friction.
- Pipeline: parse → resample to fixed spacing (e.g. every 5–10 m) → fill/smooth elevation (raw GPS
  elevation is noisy; prefer Terrain-RGB elevation from §9 where available) → compute grade with a
  smoothing window → emit `RouteModel`.
- Output: part of a **route pack** (`assets/packs/<route-id>/`) alongside terrain mesh + splats + metadata.
- This is a library with a thin CLI; the shell calls the library via FFI for in-app import, the CLI
  is for batch/offline prep.

---

## 9. Terrain substrate (`velo-terrain`)  *(Phase priority #2 — the scenery foundation)*

This is the permanent ground the rider rides on and the surface splats composite onto.

- Inputs: **raster satellite tiles** (Mapbox or equivalent) for color; **Terrain-RGB** tiles for
  elevation (or free global DEMs — Copernicus GLO-30 / FABDEM — if you want to drop the Mapbox dependency).
  (Note ToS: fine for personal use; revisit if ever distributed.)
- Pipeline (offline CLI, cached per route): for the route's bounding corridor, fetch tiles at chosen
  zoom → decode Terrain-RGB to a heightfield → generate a triangulated terrain mesh (LOD'd; finer near
  the route line, coarser at distance) → project satellite color as the texture → write to the route pack.
- Output: `TerrainTile` meshes + texture atlas, georeferenced to the route's local ENU frame.
- The renderer consumes this directly; no network at ride time.
- **Quick photorealism (Tier B, see §10):** for routes in covered cities, you can skip custom scenery
  entirely and render **Google Photorealistic 3D Tiles** over this mesh — an already-textured 3D world,
  no bake required.

---

## 10. Scene generation — tiers, simplest-first  *(read before building any of this)*

**Realism has tiers. Ship the cheap ones first; the GS bake is a *deferred upgrade*, not the starting point.**

- **Tier A — textured terrain mesh (M3).** Free/low-cost global DEM + raster or aerial tiles → a rideable,
  faithful 3D world. **No GPU preprocessing.** This is the floor and the permanent fallback everywhere.
- **Tier B — Google Photorealistic 3D Tiles, direct-render (M3b). The quick photorealism win.** For routes
  in covered cities, point the renderer at Google's already-textured 3D mesh and stream it along the route
  — **no baking, no diffusion, no GPU pipeline at all.** By far the fastest path to a genuinely photoreal
  world. **ToS guardrails (hard rules, not suggestions):** display-only via a renderer that shows
  attribution; **no caching, no offline use, no extraction/derivation** (image analysis, geodata extraction,
  measurement, and feeding tiles into any model are all prohibited). Practical consequences: (1) Tier-B
  routes are **online-only during the ride**; (2) the Tier-C bake **must never ingest Google tiles** — it
  sources its own imagery (see below). Other caveats: cities only (no rural/trails); billed Enterprise SKU
  (~1k free sessions/mo, per-session after). **Quality reality:** it's aerial-photogrammetry mesh — great
  at mid/far range, but soft/melted at cyclist eye-level (facades, ground, trees). That eye-level gap is
  exactly what Tier C fixes.
- **Tier C — VeloSplatBake (GS + FLUX), deferred.** Only when you need fidelity Google doesn't cover
  (rural routes, custom hero segments) or fully offline operation. This is the heavy, GPU-rented path
  detailed below — build it **after** A and B work and you have routes that actually need it.

**Bottom line for speed:** Tiers A+B give a photorealistic, rideable sim with *zero* GPU preprocessing.
Everything below is the long-term fidelity ceiling, not a launch requirement.

---

### VeloSplatBake (`velo-splatbake`) — Tier C detail  *(deferred; GPU-heavy; off critical path)*

Offline, GPU-heavy, per-segment. Produces 3D Gaussian splats for "hero" segments that the renderer
composites over the terrain mesh for close-up realism.

**Approach (grounded in current research):** satellite/aerial imagery gives good *coarse* geometry,
but ground-level views — exactly what the cyclist sees — are poorly constrained by satellite parallax.
The leading 2025 approach (Skyfall-GS) reconstructs coarse geometry from multi-view satellite imagery,
then runs a **general, pre-trained text-to-image diffusion model in an SDEdit-style refinement loop**
over a sky→ground curriculum to *hallucinate* photorealistic ground detail and re-optimize the splats.
Note this deliberately uses a *general* model with **no task-specific fine-tune** — that zero-shot
generality is what's doing the work. **Accept the implication:** close-up facades/ground are plausibly
synthesized, not faithful to your actual street. That's the price of having no street-level data (yet — see hook below).

**Diffusion backbone: FLUX.1-schnell.** Chosen because it's a 1–4-step distilled model (refinement is
*partial* denoising of an existing render, so steps are cheap), runs in ~12–17 GB VRAM, and is Apache-2.0
(redistributable, no licensing trap if this ever leaves the laptop). SDXL-Turbo/Lightning is the lighter
fallback (~6–8 GB, broadest ControlNet/LoRA ecosystem) if VRAM is tight. A small street-level **LoRA**
(eventually trained on real captures) is an optional later bias toward realistic facades — not required.

**Bake pipeline (CLI, offline; rent NVIDIA — see cost below):**
1. Acquire multi-view / multi-date satellite (and aerial where available) imagery for the segment.
   **Inputs must be independently sourced** (commercial/open satellite, aerial ortho/oblique, your own
   captures) — **never Google 3D Tiles** (extraction/derivation is ToS-prohibited; Tiers B and C stay
   completely walled off, sharing only the georeferenced route frame).
   *(Optional, future): ingest real ground-level views as additional supervision (see hook).*
2. Coarse GS reconstruction (RPC/affine-camera adapted 3DGS — EOGS/EOGS++/RPC-GS line of work).
3. Curriculum refinement: render views sky→ground, partial-denoise each with FLUX.1-schnell, optimize
   splats against the cleaned pseudo-ground-truth, iterate.
4. Prune → **compress (SOG or SPZ)** → fit to the route corridor → georeference to the same local ENU
   frame as the terrain mesh → write to the route pack.

**Cost/time per segment (estimate):** coarse reconstruction ~25–50 min on a 4090-class GPU, plus the
refinement loop (the dominant cost) ≈ ~1–3 h depending on view/iteration counts. Roughly $1–3 of rented
GPU per segment (A100 ~$1.10/h; spot 4090 cheaper). Chase student/research credits to drive this toward
zero. Batch overnight. *(Tighten this with Skyfall-GS's published per-segment wall-clock before committing.)*

**Dev locally on Apple Silicon, run production bakes on rented NVIDIA.** FLUX.1-schnell runs on the M4 via
**MFLUX** (MLX-native, scriptable — ideal for prototyping the SDEdit loop) or Draw Things, at ~30–50 s per
1024² image. That's fine for *developing/debugging* the pipeline without burning credits, but the M4 Air is
~3–5× slower than a 4090, and a real bake is thousands of diffusion calls per segment — so **production
bakes still go to rented NVIDIA**. Keeping FLUX.1-schnell on both sides means dev-local and prod-cloud use
the identical model. (On a 16 GB Air, FLUX needs 4-bit quant with some quality loss; SDXL-Turbo-class
distilled models are the comfortable local option if you want faster iteration at a lower ceiling.)

**Compression & the ≤40 GB storage budget.** Raw splats are 200 MB–1.5 GB/scene — unusable at scale.
Use proven open formats:
- **SOG** (PlayCanvas, open): ~20× — a 1 GB scene → ~42–55 MB.
- **SPZ** (Niantic, open, glTF-track): ~10× (~90% reduction) — 250 MB → ~25 MB. Simpler, well-supported.
- **HAC++** reports 100×+ but is research-grade — keep as a *future* lever, not a v1 dependency.

Budget math at ~40 MB/compressed hero segment: 40 GB ≈ **~1,000 segments** ≈ 100 km+ of unique baked
corridor *before* dedup. Make it fit with: hero-only splatting (mesh everywhere else), SOG/SPZ storage,
**segment dedup by spatial hash** (routes share segments — store once, reference many), and LOD tiers.
A serving backend is only needed if you abandon hero-only baking; for personal use, local packs suffice.

**Future hook — real ground-level imagery.** The bake treats real photos as *just more input views*
(step 1). When you later capture routes with Insta360 (your stated future plan), those views drop in as
direct supervision, the diffusion model shifts from "invent the street" to "fill gaps," and fidelity
climbs. Crowd-sourced third-party footage is the same mechanism but is **gated on pose recovery,
route-matching, and (mainly) copyright/licensing** — treat as a much-later, consent-based contribution
feature, not a scraping pipeline (see §18 risk).

**Outputs:** a compressed `.sog`/`.spz` payload per segment + placement transform. **Runtime cost is
rendering only** (cheap on M4); generation is strictly offline. Build this *after* M3 so there's a
renderer + ride loop to evaluate bakes in. If bakes disappoint, the mesh substrate still ships a complete product.

---

## 11. Rendering engine (`velo-render`, wgpu + Cesium Native)

**Streaming front-end: Cesium Native (`velo-cesium`).** The world's 3D Tiles (Google Photorealistic 3D
Tiles, Cesium-ion splat/mesh tilesets, terrain) are streamed by **Cesium Native** (engine-agnostic C++,
Apache-2.0) bridged via the `cxx` crate. It does the hard part — LOD selection, view-frustum culling,
caching, WGS84 math, glTF decode — and hands `velo-render` each visible tile as an in-memory glTF; it
does **no drawing itself**. We implement its three integration interfaces: `IAssetAccessor` (fetch via
our HTTP/file layer), `ITaskProcessor` (our thread pool), and `IPrepareRendererResources` (upload glTF
buffers/textures and GS payloads into wgpu resources). Picking Cesium Native over CesiumJS buys native
Metal performance and keeps the renderer in our cross-platform stack (Cesium Native + wgpu both port).

Passes (all wgpu/Metal, drawing what Cesium Native and the local packs provide):
- **Terrain pass:** `TerrainTile` meshes (Tier A) with satellite texture; sky/atmosphere; sun direction.
- **Scenery layers (best-available per segment):** Google 3D Tiles mesh (Tier B, **streamed live,
  online-only — never cached**) where covered; **SOG/SPZ** Gaussian splats (Tier C / Cesium-ion) on hero
  segments; else terrain mesh alone (Tier A).
- **Splat pass:** sorted Gaussian-splat rasterization (reuse `bevy_gaussian_splatting`'s wgpu approach);
  consumes SPZ/SOG payloads decoded by Cesium Native or loaded from local packs.
- **Foreground-object pass:** draws the rider's **bike model** (imported glTF, §11b) and any avatar —
  a normal PBR glTF mesh pass, depth-composited over the world.
- **HUD overlay pass:** rendered here, *not* in SwiftUI — must be in the 3D view, high-fps, cross-platform.
  (Liquid Glass is only the surrounding app chrome, §12.)
- Camera: chase/first-person along the route; steering offset from `SteeringInput`. Decoupled from the
  sim tick — interpolates between the latest two `RideState` snapshots.

---

## 11b. Bike model import (`velo-bikegen`)  *(rider-facing feature; offline asset-gen + foreground render)*

Turn a few 2D photos of a bike (the rider's own, or images from the web) into a 3D model shown in the
sim. This is **object-level image-to-3D**, entirely separate from scene reconstruction and far simpler:
the output is a standard textured **glTF/GLB** that the foreground-object pass (§11) already knows how to draw.

**Pipeline (offline, per bike):** gather 1–4 images (more angles → better) → run an image-to-3D model →
textured glTF → normalize scale/orientation, fit to a bike rig anchor → store in the asset library →
select as the rider's bike.

**Models:** open-source **TRELLIS.2** (Microsoft, PBR, multi-image) or **Hunyuan3D 2.1/3.x** (Tencent,
high-fidelity PBR, ~10–25 s) run locally (~16–24 GB VRAM, so rented GPU or a strong desktop); or a
**hosted API** (Meshy / Tripo / Hyper3D Rodin) for pay-per-use with no local hardware. Same "dev-local
vs cloud-prod" split as the bake applies if self-hosting.

**Caveats:**
- *Thin structures* (spokes, tubes, cables, derailleurs) are a known image-to-3D weak spot — expect a
  good overall shape but messy thin parts; multi-image input and light mesh cleanup help.
- *IP/licensing*: web product photos are copyrighted and bike designs carry brand trade dress; some
  image-to-3D model licenses restrict commercial use. Fine for personal use; revisit before distribution.

**Integration cost:** low — only the offline generation step is new; rendering reuses the existing glTF
foreground pass. Exposed to the core via a simple `BikeAsset { gltf_path, anchor_transform }` the
renderer attaches to the rider.

---

## 12. Swift platform shell (`shell-macos`)

The *only* Apple-locked code. Implements the §4 traits and owns OS chrome.

- **Liquid Glass UI (SwiftUI):** setup/pairing, route picker, workout builder, ride summary,
  settings. This is where Liquid Glass lives — menus, sheets, the surrounding app, *not* the HUD.
- **CoreBluetooth:** implements `TrainerControl` + `SensorSource` (§7), incl. state restoration/reconnect.
- **MusicKit:** implements `AudioDirector` (§13).
- **CMHeadphoneMotionManager:** implements `SteeringInput` (§15).
- **VideoToolbox:** implements `MediaCapture` (§14) — hardware H.264/HEVC encode for clips.
- **Strava API:** implements `ActivityPublisher` (§14) — OAuth2 + activity upload.
- **AppKit:** window, the wgpu-backed `CAMetalLayer`/`MTKView` surface the Rust renderer draws into, file pickers.
- Bridged to Rust via **UniFFI** (§16).

---

## 13. Audio: segment-aware MusicKit driver  *(lowest priority)*

**Hard constraint, design around it now:** Apple Music tracks are DRM-protected. MusicKit gives you
**playback control, not raw PCM** — you cannot decode catalog audio to beatmatch/crossfade it yourself.
So the trait is `AudioDirector`, not "AudioMixer."

**What it actually does:** core emits `on_segment(energy, intent)` at workout-interval boundaries
(warmup / build / threshold / recovery / cooldown → energy levels). The Swift shell maps energy to
playlists/queues, swaps tracks at boundaries, and uses MusicKit's playback controls
(play/skip/volume duck) to transition. Track selection uses whatever tempo/energy metadata is
available — best-effort, not true BPM-locked mixing. Ship this as "smart segment-aware playback,"
which is honest and still good.

---

## 14. Strava upload & media capture  *(lands at M2b — decoupled from scenery)*

The brag feature, and deliberately early (immediately after M2a) because it depends only on the ride loop
+ a framebuffer grab, not on terrain or splats.

**Ride → FIT → Strava.** The core already produces the full telemetry stream (power, cadence, HR, the
route's GPS track + elevation, time), so writing a standard **FIT** activity file is a core/library job.
Upload goes through `ActivityPublisher`: the Swift shell does Strava **OAuth2** and POSTs the FIT (plus
media) via the Strava API `/uploads` + activity endpoints. For personal use you register your own API app
and upload your own rides. **Caveat:** Strava has tightened its developer agreement (display/branding
rules, rate limits, restrictions on third-party use) — fine for personal use; read the terms before any
distribution. Keep FIT writing in Rust so a non-Strava publisher (file export, other services) is trivial later.

**Auto screenshots/video.** The renderer already draws every frame, so:
- *Screenshots* = framebuffer grab (`MediaCapture::grab_screenshot`) → PNG. **In scope for M2b.**
- *Highlight clips* = pick moments from the ride timeline (start, biggest climb, fastest segment, finish),
  run a replay camera over those frames, encode via VideoToolbox (`MediaCapture::record_clip`). Heavier
  (replay camera + encoding) → **M4**, attached to the ride summary.
- Moment selection lives in core (it owns the ride timeline); the platform-specific *encode* lives in the shell.
- **Highlight clips are M5 scope**, not M2b — M2b ships screenshot only. Core should expose a **capture-event hook** (trait/DTO) in M5 so game logic can request screenshots/clips; do not implement clip encoding until then.

---

## 14b. Ride library (internal database)  *(M2c — after M2b, before M3)*

M2b saves FIT + PNG to ad-hoc folders (`LocalRideStore`). **M2c** replaces that with a proper **ride library** the app owns long-term.

**Design:**
- New crate **`velo-rides`**: SQLite (via `rusqlite`) catalog at `~/Documents/VeloSim/rides.db` (path injectable via `Storage` trait).
- Each row: `ride_id`, `started_at_unix`, `elapsed_s`, `distance_m`, `avg_power_w`, `max_power_w`, `fit_path`, `screenshot_path`, `strava_activity_id` (nullable), `publish_status` (local/strava/failed), `route_id` (nullable, for M3+).
- Binary artifacts stay on disk beside or under a per-ride directory; DB holds metadata + paths (not blobs).
- **`Storage` trait** (`velo-platform`) gets concrete methods: `save_ride`, `list_rides`, `get_ride`, `delete_ride`.
- Rust implements DB logic; Swift shell calls via UniFFI (`RideLibrary` object or methods on `VeloHandle`).
- **Migrate** `LocalRideStore` publish path to write through `velo-rides` instead of orphan folders.
- **UI (minimal):** ride history list in sidebar — date, distance, power, publish badge; tap opens folder or Strava link.

**Future:** route packs, workout templates, and highlight clip paths attach to the same DB in M3/M5.

---

## 15. Input: AirPods steering  *(lowest priority, optional)*

- `CMHeadphoneMotionManager` provides device attitude (pitch/yaw/roll). Map yaw delta → steering axis
  in `SteerState.axis ∈ [-1, 1]`.
- Deadzone + low-pass filter + an explicit **recenter gesture** (drift is real). Only active on routes
  that support steering offset. Keyboard/gamepad implement the same `SteeringInput` trait as the default.
- Treat as a fun optional input, never a required control path.

---

## 16. Build, FFI, and tooling

- **FFI:** **UniFFI** to generate Swift bindings from the `velo-ffi` crate. Preferred over hand-written
  C ABI for the breadth of types crossing the boundary (state structs, enums, callbacks for trait impls).
  Trait impls flow Swift→Rust via UniFFI callback interfaces.
- **Telemetry stream pattern:** `SensorSource` uses a **polling model** across the FFI boundary — Swift
  pushes samples into a UniFFI callback buffer; Rust drains them each sim tick. Avoid exposing
  `std::sync::mpsc::Receiver` directly over FFI (UniFFI cannot express it cleanly). Document the chosen
  pattern in `velo-ffi` before M2a.
- **C++ FFI (Cesium Native):** the `velo-cesium` crate bridges to Cesium Native (C++) via the **`cxx`**
  crate — the one piece of non-Rust in the renderer. Build Cesium Native as a static lib (CMake) and link
  it; implement its `IAssetAccessor` / `ITaskProcessor` / `IPrepareRendererResources` on the Rust side.
- **Build:** `cargo build` produces a static lib + universal slice for Apple Silicon; an Xcode build
  phase (or `xcodebuild` script) links it (plus the Cesium Native lib) and runs `uniffi-bindgen`. Single
  `make`/`just` target builds both.
- **Rendering surface:** Rust `wgpu` draws into a `CAMetalLayer` provided by the Swift shell.
- **CI:** workspace `cargo test`; the "no Apple symbols below the shell" lint; physics golden-file replay tests.

---

## 17. Milestone plan (acceptance criteria are the agent's definition of done)

**M0 — Skeleton & boundary.**
Workspace, crates, UniFFI round-trip (Swift calls a core fn, core invokes a Swift-implemented trait).
*Done when:* a Swift button toggles a value the Rust core owns, and a fake `SensorSource` in Swift
streams samples the core logs. CI lint passes.

**M1 — Physics core (priority #1).**
`velo-units`, `Rider`, integrator (§6), ERG + SIM modes, deterministic replay.
*Done when:* golden-file replay of a recorded ride reproduces distance/time within tolerance; flat
steady-state and known-climb unit tests pass.

**M2 — Real trainer + HUD ride + Strava (priority #1).**
Split into sub-milestones to reduce integration risk:

**M2a — Trainer + HUD (done).**
CoreBluetooth FTMS, wgpu flat plane + HUD, Fake/Replay/BLE sensor modes, ERG/SIM command path.

**M2b — FIT + Strava + screenshot (done).**
Ride session recording, `velo-fit` export, framebuffer PNG, Strava OAuth/upload or local save.

**M2c — Ride library (internal database).**
`velo-rides` SQLite catalog; migrate publish/save flow off ad-hoc folders; UniFFI list/get/delete; minimal ride history UI.
*Done when:* every finished ride is indexed in the DB with metadata and artifact paths; history list shows past rides; `LocalRideStore` delegates to `velo-rides`.

**M3 — Real route + terrain substrate (priority #2).**
`velo-route-import` (GPX/FIT/RWGPS) + `velo-terrain` (raster tiles + Terrain-RGB → mesh), renderer
terrain pass, grade driving SIM resistance.
*Done when:* you ride an imported real route over satellite-textured terrain with grade-accurate resistance.

**M3b — Google 3D Tiles via Cesium Native (the quick photorealism win).**
Integrate **Cesium Native** (`velo-cesium`, `cxx`) — implement its three interfaces, feed glTF tiles to
the wgpu mesh pass. Stream Google Photorealistic 3D Tiles along the corridor where covered;
**online-only, with attribution — no caching/offline (ToS).**
*Done when:* you ride a real city route through photorealistic Google 3D Tiles streamed by Cesium Native
— **no GPU baking involved.** Fast path to a photoreal world; most routes can stop here.

**M3c — Bike model import (`velo-bikegen`).**
Image-to-3D (hosted API or self-hosted TRELLIS.2/Hunyuan3D) → normalized glTF → foreground-object pass
draws it as the rider's bike. Decoupled — can land any time after M2.
*Done when:* you import 1–4 photos of a bike and ride behind/on that 3D model in the sim.

**M4 — VeloSplatBake (GS + FLUX) — optional, deferred fidelity upgrade.**
Only for coverage gaps (rural/trails), custom hero segments, or fully-offline needs. `VeloSplatBake` CLI
(FLUX.1-schnell + SDEdit refinement) producing **SOG/SPZ-compressed** per-segment splats; renderer splat
pass; segment dedup so the ≤40 GB budget holds. *(Cesium ion managed reconstruction is the lower-effort
alternative — upload imagery, get a GS tileset Cesium Native streams.)*
*Done when:* at least one hero segment renders baked splats over the mesh at frame rate, falling back
to mesh/3D-Tiles elsewhere, within the storage budget. **Not required to ship a photorealistic product —
Tiers A+B already deliver that.**

**M5 — Workouts + Liquid Glass shell + highlight clips.**
Workout builder/engine, structured-workout ERG control, full Liquid Glass setup/summary UI, and
**highlight video clips** (replay camera + VideoToolbox encode) attached to the ride summary.
*Done when:* you can build a structured workout, ride it with auto ERG targets, and review a saved
summary with an auto-generated highlight clip.

**M6 — Apple Music + AirPods (lowest priority).**
`AudioDirector` (MusicKit segment-aware playback), `SteeringInput` (AirPods yaw → steering).
*Done when:* music shifts energy at interval boundaries and head-turn nudges steering on supported routes.

---

## 18. Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Splat bake quality/speed | High | Off critical path; mesh substrate is a complete shippable product on its own |
| Splat ground detail is hallucinated, not faithful | Medium | Set expectation up front; reserve splats for "hero" segments; mesh elsewhere; real captures later improve it |
| Storage blows the ≤40 GB budget | Medium | SOG/SPZ compression + hero-only splats + segment dedup + LOD; serving backend only as last resort |
| FTMS quirks / Wahoo-specific behavior | Medium | Target FTMS spec; test on real hardware early (M2); keep proprietary fallback documented |
| UniFFI friction with callback interfaces | Medium | Validate the trait-callback round-trip in M0 before building on it |
| Cesium Native C++ FFI / build complexity | Medium | One-time `cxx` + CMake plumbing; validate a minimal tile-stream in M3b before building on it; API is pre-1.0 (pin a version) |
| Bike image-to-3D: thin structures messy | Low | Expected for spokes/cables; use multi-image input + light cleanup; it's a cosmetic foreground asset, not load-bearing |
| Bike import IP (web photos, model licenses) | Low (personal) | Personal use OK; check model license + image rights before any distribution |
| Strava API terms / rate limits | Low (personal) | Personal app + own activities; keep FIT writing provider-agnostic; read dev agreement before distribution |
| Mining third-party Insta360 footage | High (legal) | Don't scrape. Own captures or opt-in/consented contributions only; pose + route-matching also non-trivial |
| Google 3D Tiles ToS (caching/extraction) | Medium | Render-only + online-only + attribution; never cache or feed tiles to the bake; Tiers B/C fully walled off |
| MusicKit can't truly "mix" | Low (scoped) | Designed around as `AudioDirector`; no raw-audio dependency anywhere |
| Imagery ToS if ever distributed | Low (personal) | Personal-use now; swap imagery source before any distribution |
| AirPods steering drift/fatigue | Low | Optional input; deadzone + recenter; keyboard/gamepad default |

---

## 19. Testing & validation

- **Physics:** unit tests (steady-state, climb, coast-down) + golden-file replay of recorded rides.
- **Determinism:** same route + same telemetry log + injected `Clock` → identical `RideState` trace.
- **Trait boundary:** mock implementations of every `velo-platform` trait so core is fully testable
  headless, no hardware, no Apple frameworks.
- **Hardware-in-the-loop:** a manual M2 checklist against the real Kickr (pair, ERG hold, SIM grade
  response, dropout/reconnect).
- **Render:** snapshot tests of terrain + HUD; visual review for splat passes.

---

### Open questions to resolve before M3/M3b/M4
1. Imagery/DEM provider + zoom + corridor width for the terrain mesh (Tier A).
2. Per route: does it fall in Google 3D Tiles coverage (→ Tier B, no bake) or need a Tier C bake?
3. "Hero segment" selection policy (Tier C only) — manual marking, or auto by gradient/landmark density?
4. Workout file import (Zwift `.zwo` / TrainingPeaks) or in-app builder only for v1?

---

## 20. Data sources & fusion strategy (acquisition playbook)

Realism = fusing complementary layers; diffusion only fills the residual. But **for speed, you don't
need the fusion ladder at all** — the fast path below gets a photoreal world from two free-ish layers.

### Fast path (start here — minimal inputs, no GPU bake)

| Layer | Source | Cost / License | Coverage | Notes |
|---|---|---|---|---|
| Terrain mesh | Raster/aerial tiles + Terrain-RGB **or** Copernicus GLO-30 / FABDEM | Mapbox tiers / free DEMs | Global | Tier A. The floor + permanent fallback. |
| Photoreal world | **Google Photorealistic 3D Tiles** | Billed Enterprise SKU (~1k free sessions/mo); attribution; **online-only, no caching/extraction** | Major cities only | Tier B. Direct-render, **no bake**, never a bake input. The quick win. |
| Route line | GPX/FIT/RWGPS export | User's own file | Anywhere | Already in §8. |
| Road surface | OpenStreetMap `surface` tag | ODbL (open) | Global (where tagged) | Feeds `Crr` in physics — cheap realism + accuracy. |

That's it for a shippable, photorealistic, rideable sim. Everything below is **later fidelity**, added
only for routes the fast path doesn't cover well.

### Full fusion ladder (later — for max fidelity / coverage gaps)

Fuse in this order; each layer shrinks what FLUX has to invent:

1. **Bare-earth terrain** — USGS 3DEP 1 m (US, free, no restrictions) or national LiDAR DTMs where
   available; else Copernicus/FABDEM globally. *(Best free geometry upgrade.)*
2. **Object geometry** — LiDAR **DSM** (real building/tree heights) ≫ extruded **Overture** (ODbL, ~2.6B
   footprints, many LiDAR-derived heights) / **OSM** footprints ≫ Microsoft / Google Open Buildings where sparse.
3. **Overhead appearance** — aerial orthophotos (e.g. USGS NAIP, free US) > satellite tiles; oblique
   aerial (Nearmap/Vexcel, paid) adds facade parallax; commercial multi-date satellite (Maxar/Airbus) for the GS bake.
4. **Ground-level truth (highest value)** — **Mapillary** (CC BY-SA, API w/ poses+compass, riders upload
   bike mode) and **KartaView** (CC BY-SA, 360/bike/foot); later your own **Insta360** captures. Fed to the
   bake as real supervision → shifts FLUX from "invent the street" to "fill gaps." *(This is the licensed
   version of mining ride footage; note CC BY-SA / ODbL share-alike before any distribution.)*
5. **Semantic dressing** — ESA WorldCover (10 m, free) for material/texture choice; global 1 m canopy
   height for vegetation; computed sun position for lighting; multi-date imagery for season-aware looks.
6. **Generative fill (residual only)** — FLUX.1-schnell (SDEdit), monocular depth priors, optional
   street-level LoRA.

**Licensing gradient:** open/free (OSM & Overture ODbL; USGS 3DEP no-restriction; Copernicus; WorldCover;
Mapillary & KartaView CC BY-SA) → paid/restrictive (Maxar/Nearmap; Google 3D Tiles billed + attribution;
Google Street View — off-limits for derivative use). Personal use is permissive; revisit before any release.
