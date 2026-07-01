# VeloSim — Liquid Glass UI, HUD & Zwift-Parity Guide (agent spec)

> **READ THIS FIRST. These rules override any conflicting instinct.** If a task seems to call for
> more glass, more panels, more metrics, or a custom blur — it doesn't. When unsure, do *less*.
> This document is the source of truth for every pixel of the UI layer. Do not invent components,
> do not add metrics not listed here, do not restyle Liquid Glass.

Target: **macOS 26 (Tahoe), SwiftUI, Apple Silicon.** The 3D world is a Metal view; this guide covers
the SwiftUI **interface layer** that floats above it. (This UI is platform-shell code, not portable core.)

---

## 0. The 10 hard rules (violating any = reject the output)

1. **Liquid Glass is the navigation/control/readout layer only — never the content.** The content is
   the Metal 3D world. Glass floats on top of it. Never fill large content areas with glass.
2. **Never stack glass on glass.** One glass surface per region. Glass inside glass reads as muddy sludge.
3. **Use the real API, never fake it.** `.glassEffect(...)`. Never hand-roll blur, never use
   `.background(.ultraThinMaterial)` as a stand-in, never a custom `NSVisualEffectView` for glass.
4. **Group related glass in a `GlassEffectContainer`.** Multiple loose `.glassEffect()` views without a
   container = wrong blending + bad performance.
5. **Tint selectively.** Only the single primary element in a region may be tinted (e.g. the power card
   by zone). Everything else is untinted `.regular`. No rainbow UIs.
6. **Prefer system controls.** Toolbars, tab bars, sheets, buttons, lists get Liquid Glass automatically
   on macOS 26. Use them. Only hand-build glass for the HUD cards and the ride control cluster.
7. **The HUD shows exactly the metrics in §5. No more.** No decorative widgets, no gratuitous gauges,
   no "cool" extras. Zwift's HUD is dense but disciplined; match that discipline.
8. **Numbers use monospaced digits + numericText transitions.** No layout jitter as values change.
9. **Never re-render the HUD per data packet or per sim tick.** Drive it from a throttled ~8 Hz model (§5).
10. **Honor accessibility automatically and explicitly** (Reduce Transparency, Reduce Motion, Dynamic
    Type). Legibility over the bright 3D scene is non-negotiable.

If output violates 1–3, it is fundamentally wrong and must be redone, not patched.

---

## 1. Mental model

Three stacked layers, bottom to top:

```
[ Metal 3D world ]  ← content. Never glass. Drawn by the renderer (wgpu→CAMetalLayer).
[ HUD readouts   ]  ← glass cards floating over the world (data only).
[ App chrome     ]  ← toolbars, menus, sheets (system Liquid Glass, mostly free).
```

Glass exists to let the world show through the controls, creating depth. If a surface doesn't need the
world behind it to show through, it probably shouldn't be glass — it should be a solid surface or plain content.

---

## 2. Architecture: SwiftUI glass over a Metal view

The ride screen is a `ZStack`: the Metal world at the back, the SwiftUI HUD in front. The Metal view is
hosted via `NSViewRepresentable`; the Rust/wgpu renderer draws into its `CAMetalLayer`.

```swift
import SwiftUI
import MetalKit

/// Hosts the renderer's Metal surface. The wgpu renderer draws into view.layer (CAMetalLayer).
struct WorldView: NSViewRepresentable {
    let renderer: RendererHandle          // FFI handle to the Rust renderer
    func makeNSView(context: Context) -> MTKView {
        let v = MTKView()
        v.device = MTLCreateSystemDefaultDevice()
        v.preferredFramesPerSecond = 120
        v.framebufferOnly = true
        renderer.attach(layer: v.layer as! CAMetalLayer)
        return v
    }
    func updateNSView(_ v: MTKView, context: Context) {}
}

struct RideScreen: View {
    @State private var hud: HUDModel
    let renderer: RendererHandle
    var body: some View {
        ZStack {
            WorldView(renderer: renderer).ignoresSafeArea()   // CONTENT (never glass)
            HUDOverlay(m: hud)                                 // INTERFACE (glass)
            RideControlCluster()                               // INTERFACE (glass)
        }
    }
}
```

**Rule:** SwiftUI never draws the world, and Metal never draws the HUD. Clean seam.

---

## 3. Design tokens (single source of truth — never inline magic numbers)

```swift
enum Tok {
    // 4-pt spacing grid
    static let s1: CGFloat = 4,  s2: CGFloat = 8,  s3: CGFloat = 12
    static let s4: CGFloat = 16, s6: CGFloat = 24, s8: CGFloat = 32
    // corner radii (concentric: card ⊃ tile)
    static let rCard: CGFloat = 22, rTile: CGFloat = 16
    // HUD glass spacing (morph threshold inside containers)
    static let glassGap: CGFloat = 12
}

enum Typo {
    static func bigMetric() -> Font { .system(size: 64, weight: .bold, design: .rounded) }
    static func metric()    -> Font { .system(size: 30, weight: .semibold, design: .rounded) }
    static func unit()      -> Font { .system(size: 15, weight: .semibold) }
    static func label()     -> Font { .system(size: 11, weight: .bold).width(.expanded) }
}
```

**Power-zone colors** (Coggan 7-zone, % of FTP) — the *only* place color-coding is allowed in the HUD:

```swift
enum PowerZone: Int {
    case z1 = 1, z2, z3, z4, z5, z6, z7
    static func of(watts: Int, ftp: Int) -> PowerZone {
        guard ftp > 0 else { return .z1 }
        switch Double(watts) / Double(ftp) {
        case ..<0.56: return .z1     // Active recovery ≤55%
        case ..<0.76: return .z2     // Endurance 56–75%
        case ..<0.91: return .z3     // Tempo 76–90%
        case ..<1.06: return .z4     // Threshold 91–105%
        case ..<1.21: return .z5     // VO2max 106–120%
        case ..<1.51: return .z6     // Anaerobic 121–150%
        default:      return .z7     // Neuromuscular >150%
        }
    }
    var color: Color {
        switch self {                       // Zwift-style zone palette
        case .z1: .gray;   case .z2: .blue;  case .z3: .green; case .z4: .yellow
        case .z5: Color.orange; case .z6: .red; case .z7: .purple
        }
    }
}
```

---

## 4. Correct Liquid Glass API (wrong-vs-right)

```swift
// Basic surface (default is .regular + Capsule)
.glassEffect()
.glassEffect(.regular, in: .rect(cornerRadius: Tok.rCard))
.glassEffect(.clear, in: .capsule)              // .clear = more transparent, for transient controls over media

// Tint (PRIMARY element only) + interactive (adds press bounce/shimmer)
.glassEffect(.regular.tint(zone.color.opacity(0.35)).interactive())

// Group + morph
GlassEffectContainer(spacing: Tok.glassGap) {
    Button("Camera", systemImage: "camera") {}.glassEffect().glassEffectID("cam", in: ns)
    Button("Photos", systemImage: "photo")  {}.glassEffect().glassEffectID("pho", in: ns)
}                                                // @Namespace private var ns

// Buttons: use the system styles, don't rebuild them
.buttonStyle(.glass)                            // secondary
.buttonStyle(.glassProminent)                   // primary CTA
.buttonBorderShape(.circle)                     // for icon buttons
```

```swift
// ❌ WRONG — fake glass
RoundedRectangle(cornerRadius: 20).fill(.ultraThinMaterial).blur(radius: 8)
// ✅ RIGHT
someView.glassEffect(.regular, in: .rect(cornerRadius: 20))

// ❌ WRONG — glass on glass
VStack { … }.glassEffect().padding().glassEffect()
// ✅ RIGHT — one glass surface, plain content inside it
VStack { … }.padding(Tok.s4).glassEffect(.regular, in: .rect(cornerRadius: Tok.rCard))

// ❌ WRONG — loose glass, no container (bad blend + slow)
HStack { A().glassEffect(); B().glassEffect(); C().glassEffect() }
// ✅ RIGHT
GlassEffectContainer(spacing: Tok.glassGap) { HStack { A().glassEffect(); B().glassEffect(); C().glassEffect() } }
```

---

## 5. The HUD (the disciplined core)

### 5.1 Allowed metrics — this list is exhaustive
Primary (always visible): **Power (W)**, **Cadence (rpm)**, **Heart Rate (bpm)**, **Speed (kph/mph)**,
**Distance**, **Elapsed time**, **w/kg**, **Gradient (%)**.
Contextual (only when relevant): **Lap time/number**, **Workout target vs actual** (workout mode),
**Route/elevation profile** with position marker, **rolling power graph** (last ~60 s).
**Forbidden:** anything not in this list. No "calories burned" ticker in-ride (put it in the summary),
no decorative gauges, no fake social feed.

### 5.2 Layout zones (fixed)
```
┌───────────────────────────────────────────────┐
│  [ TIME · DIST · GRADE ]  (top-center pill)     │  ← one GlassEffectContainer
│                                                 │
│                                                 │   (Metal world shows through)
│                                                 │
│  ┌── power (big, zone-tinted) ──┐               │  ← primary metric card
│  │  248 W                        │  [controls]  │
│  │  CAD 92 · HR 146 · 3.4 w/kg   │  (cluster)   │  ← secondary metrics + ride controls
│  └───────────────────────────────┘              │
│  ▁▂▃▅▇ route/elevation profile ▇▅▃ (workout bar)│  ← bottom, when relevant
└───────────────────────────────────────────────┘
```
Never center a wall of numbers over the middle of the screen — keep the world visible.

### 5.3 Update cadence (mandatory throttling)
The sim core ticks at 100 Hz and BLE packets arrive 1–4 Hz. The HUD updates at **~8 Hz** from a throttled
`@Observable` model. Never bind SwiftUI directly to the tick stream.

```swift
@Observable final class HUDModel {
    var power = 0, cadence = 0, heartRate = 0, ftp = 200
    var speed = 0.0, distanceKm = 0.0, gradient = 0.0, wattsPerKg = 0.0
    var elapsed: Duration = .zero
    var workout: WorkoutHUD? = nil          // non-nil only in workout mode (§6)
}
// A coordinator drains the core's telemetry and writes this model at ~8 Hz (every 125 ms).
```

### 5.4 HUD components (correct glass + monospaced digits)
```swift
private struct Stat: View {                      // small secondary metric
    let label: String, value: String
    var body: some View {
        VStack(spacing: 2) {
            Text(label).font(Typo.label()).foregroundStyle(.secondary)
            Text(value).font(Typo.metric()).monospacedDigit()
                .contentTransition(.numericText())
        }
        .padding(.horizontal, Tok.s3).padding(.vertical, Tok.s2)
    }
}

private struct PowerCard: View {
    let watts: Int, ftp: Int
    private var zone: PowerZone { .of(watts: watts, ftp: ftp) }
    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: Tok.s2) {
            Text("\(watts)").font(Typo.bigMetric()).monospacedDigit()
                .contentTransition(.numericText())
            Text("W").font(Typo.unit()).foregroundStyle(.secondary)
        }
        .padding(.horizontal, Tok.s4).padding(.vertical, Tok.s3)
        .glassEffect(.regular.tint(zone.color.opacity(0.30)), in: .rect(cornerRadius: Tok.rCard))
        .animation(.snappy, value: zone)          // only the tint animates, not layout
    }
}

struct HUDOverlay: View {
    let m: HUDModel
    @Namespace private var ns
    var body: some View {
        VStack {
            GlassEffectContainer(spacing: Tok.glassGap) {   // top pill
                HStack(spacing: Tok.glassGap) {
                    Stat(label: "TIME",  value: m.elapsed.hms).glassEffect(in: .capsule)
                    Stat(label: "DIST",  value: String(format: "%.1f km", m.distanceKm)).glassEffect(in: .capsule)
                    Stat(label: "GRADE", value: String(format: "%+.1f%%", m.gradient)).glassEffect(in: .capsule)
                }
            }
            Spacer()
            HStack(alignment: .bottom) {
                GlassEffectContainer(spacing: Tok.glassGap) {   // primary cluster
                    VStack(alignment: .leading, spacing: Tok.glassGap) {
                        PowerCard(watts: m.power, ftp: m.ftp)
                        HStack(spacing: Tok.glassGap) {
                            Stat(label: "CAD",  value: "\(m.cadence)").glassEffect(in: .rect(cornerRadius: Tok.rTile))
                            Stat(label: "HR",   value: "\(m.heartRate)").glassEffect(in: .rect(cornerRadius: Tok.rTile))
                            Stat(label: "W/KG", value: String(format: "%.1f", m.wattsPerKg)).glassEffect(in: .rect(cornerRadius: Tok.rTile))
                        }
                    }
                }
                Spacer()
                RideControlCluster()
            }
            if let w = m.workout { WorkoutBar(w: w) }        // §6
        }
        .padding(Tok.s6)
    }
}
```

> Legibility guard: because the world behind can be bright, keep HUD text at high contrast (white/primary),
> use `.regular` (not `.clear`) for data cards, and never rely on tint for legibility.

---

## 6. Zwift feature parity

### 6.1 Ride modes
- **Just Ride (free ride / SIM):** trainer resistance follows route grade; no targets.
- **Workout (structured / ERG):** trainer holds prescribed watts per interval; workout HUD active.
- **FTP Test:** a special workout that estimates FTP (§6.3).
- (Racing/group rides/meetups are multiplayer — **out of scope for the solo clone; do not stub fake riders.**)

### 6.2 Structured workout model (.zwo-compatible semantics)
Targets are expressed as **% of FTP**; multiply by rider FTP for the ERG watt command.

```swift
enum WorkoutBlock {
    case warmup(Duration, fromFTP: Double, toFTP: Double)     // ramp up
    case steady(Duration, atFTP: Double)                       // ERG hold
    case intervals(reps: Int, on: Duration, onFTP: Double, off: Duration, offFTP: Double)
    case ramp(Duration, fromFTP: Double, toFTP: Double)
    case freeRide(Duration)                                    // NO erg target — steady flat resistance
    case cooldown(Duration, fromFTP: Double, toFTP: Double)
}
struct Workout { let name: String; let blocks: [WorkoutBlock] }

struct WorkoutHUD {                     // what the HUD shows in workout mode
    let targetWatts: Int, actualWatts: Int
    let intervalRemaining: Duration
    let blockName: String
    let nextBlockName: String?
}
```
Workout engine responsibilities: walk the block timeline, compute `targetWatts = ftp * pct`, send it via
`TrainerControl.set_target_power` (ERG) except in `freeRide` (send SIM/steady), expose `WorkoutHUD`, and
fire an "interval change" cue 5 s before each transition.

### 6.3 FTP tests (exact protocols — do not improvise numbers)

| Test | Warm-up | Ramp | FTP formula | For |
|---|---|---|---|---|
| **Ramp Test** | ~5 min free | start **100 W**, **+20 W / min** to exhaustion | **75% of best 1-min avg power** | experienced/heavier riders |
| **Ramp Test Lite** | ~5 min free | start **50 W**, **+10 W / min** | **75% of best 1-min avg power** | <60 kg / beginners / FTP < ~175 W |
| **FTP Test (20-min)** | long primer + 5-min effort | — (20-min max effort in a `freeRide` block) | **95% of 20-min avg power** | experienced, best accuracy |
| **FTP Test (shorter)** | short primer | — same 20-min effort | **95% of 20-min avg** | as above, less time |

Ramp-test engine (drives ERG upward, detects failure, computes FTP):

```swift
@Observable final class RampTestEngine {
    let startWatts: Int, stepWatts: Int
    private(set) var target: Int
    private(set) var secs = 0
    private var window: [Double] = []          // last 60 s of power
    private(set) var best1MinAvg = 0.0
    private let previousFTP: Int

    init(lite: Bool, previousFTP: Int) {
        startWatts = lite ? 50 : 100
        stepWatts  = lite ? 10 : 20
        target = startWatts; self.previousFTP = previousFTP
    }

    /// Call once per second with the rider's actual power; returns the ERG target to command.
    func tick(power: Double, cadence: Double) -> (target: Int, failed: Bool) {
        secs += 1
        window.append(power); if window.count > 60 { window.removeFirst() }
        best1MinAvg = max(best1MinAvg, window.reduce(0, +) / Double(window.count))
        if secs % 60 == 0 { target += stepWatts }               // step every minute
        let failing = cadence < 50 || power < Double(target) * 0.70   // can't hold the step
        return (target, failing)
    }

    func finish() -> (ftp: Int, changed: Bool) {
        let ftp = Int((best1MinAvg * 0.75).rounded())
        return (ftp, ftp != previousFTP)
    }
}
```
For the 20-min test: run the warm-up as ERG, the 20-min block as `freeRide`, capture the 20-min average,
`ftp = round(avg * 0.95)`. On completion of any test: if higher, show a **"New FTP set!"** glass sheet
with old→new, and recompute zones + w/kg-based category.

### 6.4 Zones, categories, records
- Training zones from FTP (§3 palette). Race category by FTP-in-w/kg (A/B/C/D bands) — display only.
- Track power PRs (5s/1min/5min/20min best), and TSS/IF for the ride if FTP is known (show in summary, §7.6).

---

## 7. Screens & dashboards (Zwift-equivalent, spec'd)

Every screen: system nav (`NavigationStack`/toolbar = free Liquid Glass), content in plain surfaces,
glass only for floating controls/CTAs. **Do not reskin the nav.**

### 7.1 Home / Dashboard
Purpose: launch a ride, see identity + recent history. Layout (a scrollable content page, NOT glass):
- **Profile header:** name, FTP (tap → test), weight, w/kg. Plain card.
- **Quick start row:** big glass-prominent buttons — `Just Ride`, `Workout`, `FTP Test`, `Route`.
- **Recent activities:** list of past rides (date, distance, time, avg power, map thumb). Standard `List`.
- **Lifetime stats:** total distance / time / elevation. Plain tiles.
- **Do NOT build:** XP bar, level, drops, "Ride On" feed, challenges — those are multiplayer/gamification.
  If a light personal-progression element is wanted, it must be explicitly requested; default is none.

```swift
struct QuickStart: View {
    var body: some View {
        GlassEffectContainer(spacing: Tok.s3) {
            HStack(spacing: Tok.s3) {
                Button("Just Ride", systemImage: "bicycle") {}.buttonStyle(.glassProminent)
                Button("Workout",   systemImage: "list.bullet.rectangle") {}.buttonStyle(.glass)
                Button("FTP Test",  systemImage: "gauge.high") {}.buttonStyle(.glass)
                Button("Route",     systemImage: "map") {}.buttonStyle(.glass)
            }
        }
    }
}
```

### 7.2 Pairing
Purpose: connect sensors before a ride. One row per device role. Plain list; per-row a glass "connect"
control. Roles: **Power/Trainer (controllable)**, **Cadence**, **Heart Rate**. Show connection state
(searching / connected + battery). A prominent glass `Ride` CTA enabled once the trainer is paired.

```swift
struct PairRow: View {
    let role: String, device: String?, connected: Bool
    var body: some View {
        HStack {
            Label(role, systemImage: connected ? "checkmark.circle.fill" : "dot.radiowaves.left.and.right")
                .foregroundStyle(connected ? .green : .secondary)
            Spacer()
            Text(device ?? "Searching…").foregroundStyle(.secondary).monospacedDigit()
            Button(connected ? "Change" : "Connect") {}.buttonStyle(.glass).controlSize(.small)
        }
        .padding(Tok.s3)
    }
}
```

### 7.3 Mode / route select
- Mode picker → for Just Ride/route: **route list with elevation profile, distance, total ascent**.
- Show the route's elevation profile as a small sparkline per row; full profile on the detail.
- Note per route whether it's **Google 3D Tiles coverage** (photoreal) or terrain-only (from the plan).

### 7.4 Workout library
- Folders: **FTP Tests**, plus user workouts. Each workout shows the **interval graph** (colored blocks
  scaled by %FTP and duration) + total duration + TSS estimate. Detail view = big interval graph + Start.

### 7.5 In-ride (HUD §5 +)
Add these Zwift in-ride elements, all glass, all optional-by-context:
- **Route/elevation profile bar** (bottom): the climb profile with a moving position dot + gradient-colored fill.
- **Workout interval bar** (workout mode): upcoming blocks, current block highlighted, target vs actual
  power delta, countdown to next interval, ± ERG bias buttons, skip-interval.
- **Ride control cluster** (bottom-right glass container): pause, camera view toggle, screenshot, U-turn.
- **Lap** button + auto-lap. **Rolling power graph** (~60 s) optional, top-right, small.
- **No rider list / leaderboard** (multiplayer).

```swift
struct WorkoutBar: View {
    let w: WorkoutHUD
    var body: some View {
        GlassEffectContainer(spacing: Tok.glassGap) {
            HStack(spacing: Tok.s4) {
                VStack(alignment: .leading) {
                    Text(w.blockName).font(Typo.label())
                    Text("\(w.actualWatts) / \(w.targetWatts) W").font(Typo.metric()).monospacedDigit()
                        .foregroundStyle(abs(w.actualWatts - w.targetWatts) <= 10 ? .green : .primary)
                }
                Spacer()
                Text(w.intervalRemaining.mmss).font(Typo.metric()).monospacedDigit()
            }
            .padding(Tok.s4)
            .glassEffect(.regular, in: .rect(cornerRadius: Tok.rCard))
        }
    }
}
```

### 7.6 Pause menu & end-of-ride summary
- **Pause overlay:** dim the world slightly, a centered glass card: Resume / End Ride / Discard.
- **Summary:** distance, moving time, elevation gain, avg+max power, normalized power, avg+max HR,
  avg cadence, calories, TSS/IF (if FTP known), any power PRs, **FTP change** (if a test), route map,
  and **Save + Upload to Strava** / Discard. Content page with plain cards; glass only on the CTAs.

---

## 8. Accessibility (mandatory, not optional)

```swift
@Environment(\.accessibilityReduceTransparency) private var reduceTransparency
@Environment(\.accessibilityReduceMotion)       private var reduceMotion
```
- **Reduce Transparency:** swap glass data cards for a solid high-contrast surface. Provide one helper and
  use it everywhere HUD legibility matters:
```swift
extension View {
    @ViewBuilder func hudSurface(_ shape: some Shape, reduceTransparency: Bool) -> some View {
        if reduceTransparency { self.background(Color.black.opacity(0.65), in: shape) }
        else { self.glassEffect(.regular, in: shape) }
    }
}
```
- **Reduce Motion:** disable `.interactive()` bounce, `glassEffectTransition(.materialize)`, and morph
  animations; use plain fades.
- **Dynamic Type:** metrics use fixed sizes for the big HUD numbers (they must not reflow), but labels and
  all chrome respect Dynamic Type.
- **VoiceOver:** every HUD metric has an `.accessibilityLabel` ("Power, 248 watts").
- **Contrast:** HUD text is always `.primary`/white over the scene; never rely on tint or thin glass for contrast.

---

## 9. Performance rules

- HUD model updates at **~8 Hz**; never bind to the 100 Hz tick or per-BLE-packet.
- One `GlassEffectContainer` per HUD region; do **not** scatter dozens of independent glass views.
- Animate **only** value/tint changes (`.numericText`, tint), never layout, never per-frame.
- No `.blur`, no `.shadow` stacks faking depth — glass provides depth.
- Avoid `.drawingGroup()` over the Metal view (it rasterizes and breaks glass sampling of the world).

---

## 10. Anti-pattern catalog (the "crap" to reject on sight)

- ❌ Glass on every card, full-screen glass, glass on the scrollable content page → ✅ glass only on floating controls/HUD.
- ❌ Glass-on-glass nesting → ✅ one glass surface, plain content inside.
- ❌ `.ultraThinMaterial` / custom blur pretending to be glass → ✅ `.glassEffect`.
- ❌ Rainbow tints everywhere → ✅ tint only the primary element (power by zone).
- ❌ Loose `.glassEffect` views, no container → ✅ `GlassEffectContainer`.
- ❌ HUD re-rendering per packet, jittery non-monospaced numbers → ✅ 8 Hz model + monospacedDigit + numericText.
- ❌ Inventing metrics/gauges, fake rider lists, XP/drops for a solo app → ✅ only §5/§7 elements.
- ❌ Made-up FTP-test numbers → ✅ the exact protocols in §6.3.
- ❌ Reskinning the toolbar/tab bar/sheets → ✅ use system controls; they're already Liquid Glass.
- ❌ Centering a wall of numbers over the world → ✅ the zoned layout in §5.2.

---

## 11. Compliance checklist (agent must self-verify before "done")

- [ ] No `.ultraThinMaterial`/custom blur anywhere; all glass via `.glassEffect`.
- [ ] No glass-on-glass; no full-screen/content glass.
- [ ] Every multi-element glass region wrapped in `GlassEffectContainer`.
- [ ] Only the power card is tinted (by zone); everything else `.regular`.
- [ ] HUD shows only §5 metrics; layout matches §5.2 zones.
- [ ] HUD bound to an ~8 Hz `@Observable` model, not the tick/packet stream.
- [ ] All numbers `.monospacedDigit()` + `.contentTransition(.numericText())`.
- [ ] FTP tests use the exact §6.3 protocols and formulas.
- [ ] Workout targets are %FTP → watts; free-ride blocks send SIM, not ERG.
- [ ] Reduce Transparency, Reduce Motion, Dynamic Type, VoiceOver all handled.
- [ ] System nav/toolbars/sheets used as-is (not reskinned).
- [ ] No multiplayer/gamification stubs (no fake riders, XP, drops, leaderboards).

If any box is unchecked, the UI is not done.
