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

1. **Home** — rider card, Next Ride hero, quick start, lifetime stats.
2. **Just Ride** — instant free ride on open terrain: chase camera, glass
   HUD (3 s power, zone-tinted card, sparkline), pause menu (Esc/Space).
3. **Activities → Routes** — import any GPX (or use an installed pack);
   sparkline previews, pre-ride readiness rail, Start ride. Note the road
   band and elevation bar following the real profile.
4. **Activities → Workouts** — start the 2x20 Threshold: workout bar with
   target/countdown/progress, ERG bias ±, skip interval, interval-change
   flash banner.
5. **FTP Test** (Home quick start) — ramp test steps ERG upward and
   announces the new FTP when you fade; the 20-min protocol is rider-paced.
6. **End ride** — summary sheet: tinted stat tiles, NP/IF/TSS, elevation
   gain; ride lands in History and (if connected) uploads to Strava.

## Optional connections

- **Strava** and **Apple Music** connect from Settings via guided wizards.
- **Photorealistic 3D tiles** need a Google Map Tiles or Cesium ion key in
  Settings → Integrations; without keys the synthetic terrain is used.

## Screenshot mode (CI / review)

`swift run VeloSim --screenshots /tmp/shots` renders Home, Activities,
Settings, the in-ride HUD, and the ride summary to PNGs headlessly and
exits — the same path CI uses for visual review.
