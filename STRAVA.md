# Strava setup (VeloSim M2b)

Personal-use Strava upload uses OAuth2 PKCE from the Swift shell. FIT encoding stays in Rust (`velo-fit`); HTTP upload stays in Swift.

## 1. Register a Strava API application

1. Go to [https://www.strava.com/settings/api](https://www.strava.com/settings/api)
2. Create an application (any name; **Authorization Callback Domain** can be `localhost` — we use a custom URL scheme in the app)
3. Note **Client ID** and **Client Secret**

## 2. Configure credentials (pick one)

### Option A — environment variables (recommended for dev)

```bash
export STRAVA_CLIENT_ID="your_client_id"
export STRAVA_CLIENT_SECRET="your_client_secret"
./scripts/build.sh
shell-macos/.build/release/VeloSim
```

### Option B — plist (local only, do not commit)

Copy `shell-macos/StravaConfig.example.plist` → `shell-macos/StravaConfig.plist` and fill in values. Add the plist to the app bundle in your Xcode/copy step if you use Xcode; for `swift build`, prefer env vars.

## 3. OAuth redirect

VeloSim uses custom URL scheme **`velosim://oauth`**.

- Registered in `shell-macos/Info.plist` (`CFBundleURLTypes`)
- After browser auth, Strava redirects to `velosim://oauth?code=…` and the app exchanges the code via PKCE

In the Strava app settings, set **Authorization Callback Domain** to a value Strava accepts (often `localhost`); the redirect URI sent in the authorize request is still `velosim://oauth`.

## 4. Ride flow

1. **Start ride** — core `RideSession` records per-tick telemetry
2. **Stop & publish** — Rust exports FIT, captures framebuffer RGBA, Swift encodes PNG
3. If Strava is configured **and** tokens exist in Keychain → `POST /api/v3/uploads` with FIT + screenshot
4. Otherwise → save to **`~/Documents/VeloSim/rides/<ISO8601>/`** (`ride.fit`, `screenshot.png`, `summary.json`)

## 5. Indoor GPS placeholder

Without a real route (M3), FIT files use a fixed indoor coordinate pair (Montreal velodrome placeholder — see `velo-fit::INDOOR_LAT_DEG` / `INDOOR_LON_DEG`). Strava will show a short indoor activity with power/cadence/HR records.

## 6. Tests

```bash
cargo test                    # Rust: velo-fit round-trip, ride session, integration
cd shell-macos && swift test  # Swift: OAuth, upload multipart, PNG, ride FFI
./scripts/build.sh            # Full release build + UniFFI regen
```

Never commit real tokens or `StravaConfig.plist` with secrets.
