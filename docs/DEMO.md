# VeloSim demo quickstart

Build and run the native macOS app from a fresh checkout, then walk the
five-minute demo flow. Requires Xcode command line tools and Rust (stable).

## Build & run

```sh
# 1. Build the Rust core once (release).
cargo build --release -p velo-ffi

# 2. Generate the Swift bindings.
cargo run -p velo-ffi --bin uniffi-bindgen -- generate \
  --library target/release/libvelo_ffi.dylib \
  --language swift \
  --out-dir shell-macos/Generated
cp shell-macos/Generated/velo_ffiFFI.h shell-macos/Bridge/include/velo_ffiFFI.h

# 3. Run the app.
cd shell-macos && swift run VeloSim
```

No trainer required — the default **Replay** input simulates a rider, and
**Fake** input gives manual control. A Bluetooth FTMS trainer works out of
the box via **BLE (FTMS)** in the pre-ride panel.

## Five-minute demo flow

1. **Home** — rider card, Next Ride hero (hover it), colorful quick-start
   tiles, pinned route/workout with interval-shape preview, lifetime stats.
2. **Just Ride** — instant free ride on open terrain: sunlit sky, chase
   camera, glass HUD (3 s power, zone-tinted card, sparkline), pause menu
   (Esc/Space). Toggle minimal HUD mode for the pared-down pill layout.
3. **Activities → Routes** — import any GPX (or use an installed pack);
   terrain-filled sparklines with distance chips, pre-ride readiness rail,
   Start ride. Note the marked road band, roadside trees/bushes/rocks, and
   the elevation bar following the real profile — the ridden part shades
   brighter behind the position dot, and rides loop the course past the
   end.
4. **Activities → Workouts** — start the 2x20 Threshold: the pre-ride rail
   previews the armed workout (duration-weighted zone bars, TSS) next to
   Start; in-ride, the workout bar shows target/countdown/progress and the
   next-interval hint, with ERG bias ±, skip interval, and interval-change
   flash banner.
5. **FTP Test** (Home quick start) — ramp test steps ERG upward and
   announces the new FTP when you fade; the 20-min protocol is rider-paced.
6. **End ride** — summary sheet: tinted stat tiles, NP/IF/TSS, elevation
   gain, and detected highlight moments; ride lands in History (totals
   strip, zone-tinted rows) and (if connected) uploads to Strava.
7. **Bike import** (Activities → Routes → Bike) — 1–4 photos generate a
   placeholder bike tinted from your photos; the sampled frame color shows
   as a swatch and rides with you.

## Optional connections

- **Strava** and **Apple Music** connect from Settings via guided wizards.
- **Photorealistic 3D tiles** need a Google Map Tiles or Cesium ion key in
  Settings → Integrations; without keys the synthetic terrain is used.

## Screenshot mode (CI / review)

`swift run VeloSim --screenshots /tmp/shots` renders Home, Activities,
Settings, History, the workout builder, the in-ride HUD (full and
minimal), and the ride summary to PNGs headlessly and exits — the same
path CI uses for visual review. The Rust scene renders headlessly via
`cargo run -p velo-eval-mcp -- call render_frame` (see `list-tools`).
