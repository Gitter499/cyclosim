#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REF="https://github.com/Gitter499/cyclosim/blob/dev/VeloSim-Technical-Plan.md"

create() {
  local title="$1"
  local body_file="$2"
  shift 2
  local url
  url=$(gh issue create -t "$title" -F "$body_file" "$@")
  echo "$url"
}

close_done() {
  local url="$1"
  local num="${url##*/}"
  gh issue close "$num" --comment "Implemented on \`dev\` (see \`initial-monolith\` tag and subsequent commits)."
  echo "closed #$num"
}

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

cat >"$tmpdir/m0.md" <<EOF
## Acceptance criteria
Workspace, crates, UniFFI round-trip (Swift calls a core fn, core invokes a Swift-implemented trait).

**Done when:** a Swift button toggles a value the Rust core owns, and a fake \`SensorSource\` in Swift streams samples the core logs. CI lint passes.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m1.md" <<EOF
## Acceptance criteria
\`velo-units\`, \`Rider\`, integrator (§6), ERG + SIM modes, deterministic replay.

**Done when:** golden-file replay of a recorded ride reproduces distance/time within tolerance; flat steady-state and known-climb unit tests pass.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m2a.md" <<EOF
## Acceptance criteria
CoreBluetooth FTMS, wgpu flat plane + HUD, Fake/Replay/BLE sensor modes, ERG/SIM command path.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m2b.md" <<EOF
## Acceptance criteria
Ride session recording, \`velo-fit\` export, framebuffer PNG, Strava OAuth/upload or local save.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m2c.md" <<EOF
## Acceptance criteria
\`velo-rides\` SQLite catalog; migrate publish/save flow off ad-hoc folders; UniFFI list/get/delete; minimal ride history UI.

**Done when:** every finished ride is indexed in the DB with metadata and artifact paths; history list shows past rides; \`LocalRideStore\` delegates to \`velo-rides\`.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m3.md" <<EOF
## Acceptance criteria
\`velo-route-import\` (GPX/TCX) + \`velo-terrain\` (synthetic/Copernicus DEM → mesh), renderer terrain pass, grade driving SIM resistance.

**Done when:** you ride an imported real route over satellite-textured terrain with grade-accurate resistance.

## Follow-ups (deferred from initial M3)
- Copernicus GLO-30 tile fetch behind \`network-fetch\`
- FIT route import
- Real satellite raster tiles
- Terrain LOD refinement

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m3b.md" <<EOF
## Acceptance criteria
Integrate **Cesium Native** (\`velo-cesium\`, \`cxx\`) — implement its three interfaces, feed glTF tiles to the wgpu mesh pass. Stream Google Photorealistic 3D Tiles along the corridor where covered.

**ToS guardrails:** online-only, with attribution — no caching/offline.

**Done when:** you ride a real city route through photorealistic Google 3D Tiles streamed by Cesium Native — no GPU baking involved.

## Prerequisites
- M3 route packs and ENU frame
- Pin a specific Cesium Native release (API pre-1.0)
- Validate minimal tile-stream in spike before full integration

## Reference
[VeloSim-Technical-Plan.md §10 Tier B, §17]($REF)
EOF

cat >"$tmpdir/m3c.md" <<EOF
## Acceptance criteria
Image-to-3D (hosted API or self-hosted TRELLIS.2/Hunyuan3D) → normalized glTF → foreground-object pass draws it as the rider's bike.

**Done when:** you import 1–4 photos of a bike and ride behind/on that 3D model in the sim.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m4.md" <<EOF
## Acceptance criteria
Only for coverage gaps (rural/trails), custom hero segments, or fully-offline needs. \`VeloSplatBake\` CLI (FLUX.1-schnell + SDEdit refinement) producing SOG/SPZ-compressed per-segment splats; renderer splat pass; segment dedup so the ≤40 GB budget holds.

**Done when:** at least one hero segment renders baked splats over the mesh at frame rate, falling back to mesh/3D-Tiles elsewhere, within the storage budget.

**Note:** Not required to ship a photorealistic product — Tiers A+B already deliver that.

## Reference
[VeloSim-Technical-Plan.md §10 Tier C, §17]($REF)
EOF

cat >"$tmpdir/m5.md" <<EOF
## Acceptance criteria
Workout builder/engine, structured-workout ERG control, full Liquid Glass setup/summary UI, and **highlight video clips** (replay camera + VideoToolbox encode) attached to the ride summary.

**Done when:** you can build a structured workout, ride it with auto ERG targets, and review a saved summary with an auto-generated highlight clip.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

cat >"$tmpdir/m6.md" <<EOF
## Acceptance criteria
\`AudioDirector\` (MusicKit segment-aware playback), \`SteeringInput\` (AirPods yaw → steering).

**Done when:** music shifts energy at interval boundaries and head-turn nudges steering on supported routes.

## Reference
[VeloSim-Technical-Plan.md §17]($REF)
EOF

create_and_close() {
  local title="$1"
  local body_file="$2"
  local url
  url=$(create "$title" "$body_file" -l milestone -l done)
  close_done "$url"
}

create_and_close "M0: Skeleton and boundary" "$tmpdir/m0.md"
create_and_close "M1: Physics core" "$tmpdir/m1.md"
create_and_close "M2a: Trainer and HUD" "$tmpdir/m2a.md"
create_and_close "M2b: FIT, Strava, and screenshot" "$tmpdir/m2b.md"
create_and_close "M2c: Ride library" "$tmpdir/m2c.md"
create_and_close "M3: Real route and terrain substrate" "$tmpdir/m3.md"

create "M3b: Google 3D Tiles via Cesium Native" "$tmpdir/m3b.md" -l milestone
create "M3c: Bike model import (velo-bikegen)" "$tmpdir/m3c.md" -l milestone
create "M4: VeloSplatBake (optional splat fidelity)" "$tmpdir/m4.md" -l milestone
create "M5: Workouts, Liquid Glass shell, highlight clips" "$tmpdir/m5.md" -l milestone
create "M6: Apple Music and AirPods steering" "$tmpdir/m6.md" -l milestone

gh issue list --state all --limit 20
