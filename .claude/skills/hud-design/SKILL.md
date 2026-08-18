---
name: hud-design
description: >
  Design doctrine for VeloSim's in-ride HUD. Use whenever working on HUD, overlay,
  or in-ride UI in this repo: the SwiftUI overlay
  (shell-macos/Sources/VeloSim/UI/HUD/RideHUDOverlay.swift and UI/Design tokens)
  or the Rust wgpu/glyphon HUD (crates/velo-render/src/hud.rs) used for
  screenshots, highlight clips, and velo-eval renders. Covers layout regions,
  type scale, power-zone palette, number-display rules, and contrast on
  translucent panels. Both HUDs must follow this single design language.
---

# VeloSim HUD Design Doctrine

The HUD is read at glance speed while the rider is under physical load, exactly like a
racing-game HUD or a bike head unit. Every rule below follows from that: peripheral
vision reads contrast, color, and position — not fine typography — and the center of
the screen belongs to the road.

## 1. Layout doctrine — named regions

```
+--------------------------------------------------------------+
| TOP STRIP: time · speed · dist · grade (· lap)               |
| ELEVATION BAR: route profile + rider dot (top, under strip)  |
|                                                              |
|                     ROAD (keep clear)                        |
|                     no HUD in center                         |
|                                                              |
| PRIMARY BLOCK          |                    CONTROL CLUSTER  |
|  power (zone-colored)  |                    (interactive     |
|  power sparkline       |                     buttons only)   |
|  CAD · HR · W/KG       |                                     |
| WORKOUT BAR: interval name · target W · time left · progress |
| attribution (small, dim)                                     |
+--------------------------------------------------------------+
```

Region contract — what belongs where, and nothing else:

- **Primary block (bottom-left).** One hero metric only: current power, 3s-smoothed,
  in the largest type on screen, surface tinted by power zone. Secondary live metrics
  (cadence, HR, W/kg) sit directly beneath it in one row, same panel group. Zwift,
  TrainerRoad, and head units all anchor power + HR + cadence in a single corner
  cluster; do not scatter them.
- **Top strip (top-center pill).** Ambient ride state: elapsed time, speed, distance,
  grade, lap. These are "context" metrics glanced a few times a minute — smaller type,
  one horizontal pill, never competing with the primary block.
- **Elevation bar (top, below the strip).** Route silhouette + position dot. It is a
  map, not a metric: low-contrast fill, one accent dot. Never taller than ~40 px.
- **Workout bar (bottom, full-width-ish).** Only when a workout is active: interval
  name, target watts, time remaining, progress fill, upcoming-interval hint. Target vs.
  actual belongs here, not in the primary block.
- **Control cluster (bottom-right).** The only interactive region. Buttons never mix
  into metric panels; metrics are `allowsHitTesting(false)`.
- **Center of screen: empty.** No panels, no persistent text. Transient events (lap
  banner, interval change) may flash near — not at — center, then leave within ~2 s.
- **Panel count:** at most 4 visible groups in free ride (top strip, elevation,
  primary block, controls), 5 with the workout bar. Minimal mode collapses to the
  power card alone. New metrics go inside an existing group or they don't ship.

## 2. Type scale (ratios, not px)

Anchor everything to the hero power numeral = **1.0**:

| Role | Ratio | Weight | Existing token |
|---|---|---|---|
| Primary metric (power) | 1.0 (64 pt in-app) | bold, rounded | `Typo.bigMetric()` |
| Secondary metrics (top strip, CAD/HR/W-kg) | ~0.45–0.5 | semibold | `Typo.metric()` |
| Units ("W", "km/h") | ~0.22–0.25 | semibold | `Typo.unit()` |
| Labels ("CAD", "HR") | ~0.17, UPPERCASE, tracked wide | bold | `Typo.label()` |
| Footnotes (attribution) | ≤0.15, ~70% opacity | regular | `.caption2` |

Rules:
- Exactly one metric at ratio 1.0. If two things are "most important," one of them isn't.
- Labels are uppercase, letter-spaced, and dimmer than values (secondary foreground).
  The value carries the contrast; the label is furniture.
- Units render at unit scale next to the numeral, baseline-aligned, dimmer — never at
  numeral size ("312 W", not "312W" in one style).

## 3. Numbers that update live

- **Tabular / fixed-width digits always.** Proportional figures make "111" narrower
  than "888" and the layout jitters every tick. SwiftUI: `.monospacedDigit()` (already
  in place — keep it on every live numeral). Rust: monospace family, or right-align
  each numeral run against a fixed edge so width changes grow leftward invisibly.
- **Fixed slots, no reflow.** Reserve width for the worst case (4 digits of power,
  "888.8" speed). A value change must never move any other element. Placeholders use
  an em dash "—" in the same slot, never collapse the tile.
- **3-second power smoothing.** Never display raw instantaneous watts — every head
  unit and every serious app shows a rolling 3 s average because raw power is too
  noisy to read or pace from. Smooth in the model, not per-view, so both HUDs and
  exports agree. (Zone tinting uses the same smoothed value — no flickering zone color.)
- **Update cadence:** numerals at ≤ 8 Hz (the existing `HUDCoordinator` throttle);
  1–4 Hz is plenty for anything but power. Gauges/bars may animate at frame rate.
- **Motion restraint.** No slides, bounces, or pulsing on metric changes. SwiftUI's
  `.contentTransition(.numericText())` is the maximum allowed flourish, and it must be
  disabled under Reduce Motion. Zone-color changes crossfade ≤ 250 ms.

## 3b. Transient events (interval change, lap, FTP update)

- **One event surface.** A single banner fades in near — not at — center
  (upper third, below the elevation bar), announces, and leaves. Never two
  transient surfaces at once; a newer event replaces the current one.
- **Content: one line.** Event name plus the one number that matters
  ("Threshold 2 · 250 W", "Lap 3"). Same surface treatment as permanent
  panels (scrim ≥ 0.45, rounded card); an accent may edge the banner but
  zone color never floods it.
- **Timing:** fade in ≤ 250 ms, hold ~1.6 s, fade out ≤ 400 ms — gone within
  ~2 s. Fade only: no slides, scales, or bounces. Under Reduce Motion the
  banner appears and disappears without animation.
- **Model-driven triggering.** The event is raised where state changes
  (interval index, lap count), not inferred per-view, so both HUDs and
  replays agree on when a flash happened. Expiry is also model-side (clear
  after ~2 s on the next tick) — views never own timers for it.

## 4. Power-zone palette (Coggan 7-zone)

Boundaries match `PowerZone.of(watts:ftp:)` in
`shell-macos/Sources/VeloSim/UI/Design/PowerZone.swift`. These hexes are the canonical
VeloSim values — tuned to stay distinguishable on a dark translucent panel and to keep
white text readable when used as a tinted surface.

| Zone | Name | %FTP | Hex | Role color |
|---|---|---|---|---|
| Z1 | Active Recovery | < 55% | `#9AA5B1` | grey |
| Z2 | Endurance | 55–75% | `#3D9BE9` | blue |
| Z3 | Tempo | 76–90% | `#3FBE58` | green |
| Z4 | Threshold | 91–105% | `#F5C542` | yellow |
| Z5 | VO2max | 106–120% | `#F07F2E` | orange |
| Z6 | Anaerobic | 121–150% | `#E43F4F` | red |
| Z7 | Neuromuscular | > 150% | `#B05CE0` | purple |

Usage rules:
- Zone color is used as a **surface tint or accent** (panel tint at 25–45% over the
  dark backdrop, edge bar, gauge fill) — not as the text color of the numeral. The
  numeral stays white; the zone reads peripherally from the colored surface. (Full-
  saturation Z4 yellow behind white text fails contrast; tint, don't flood.)
- Zone color-coding is the **only** semantic color on the HUD. Don't add a second
  color code (e.g., red = warning) that collides with Z6.
- Zone bars/graphs use these same hexes; the FTP reference line is neutral white 25%.

## 5. Contrast on translucent panels over variable scenes

The scene behind the HUD ranges from night asphalt to snow and open sky, so contrast
must be computed against the worst case (white background), not the average.

- **Effective contrast target:** ≥ 4.5:1 for secondary text/labels, ≥ 3:1 for the
  hero numeral (large text) — measured as white text vs. (backdrop composited over
  pure white scene).
- **Backdrop alpha guidance:** a neutral-dark scrim needs ≥ ~0.45 alpha before white
  text survives a snow/sky scene; 0.55–0.80 is the comfortable band. The Rust HUD's
  panel `[0.03, 0.05, 0.08, 0.78]` (dark blue-grey @ 78%) is a good reference; do not
  drop panels below ~0.35 alpha, ever. SwiftUI glass materials get a dark tint layer
  for the same reason — material blur alone is not contrast.
- Text never renders directly on the scene. Everything sits on a panel/scrim; small
  footnote text (attribution) that must float gets a subtle shadow or local scrim.
- Respect Reduce Transparency: swap glass for solid dark fills (already plumbed via
  `reduceTransparency` — keep every new surface on that path).

## 6. Do / Don't

Do:
- Keep dead-center clear; pin everything to edges and corners.
- One hero metric (power); everything else at least a step down the scale.
- Group related metrics into shared panels (chunking) — a glance reads a panel, not
  seven scattered tiles. Target ≤ ~5 values per panel, ~4 panels total.
- Use consistent units everywhere (km/h, km, m, W, bpm, rpm, W/kg) and show the unit
  once per value, small and dim.
- Right-align numeric columns; left-align text; keep label-above-value orientation
  consistent across all tiles.
- Use the zone palette for power everywhere power is colored (numeral surface, sparkline,
  workout target, zone bars) — one palette, both HUDs.
- Keep interactive controls in their own cluster with real hit targets; metrics are
  non-interactive.
- Provide a minimal mode (power card only) — decluttering is a feature riders expect.

Don't:
- Don't center-stack metrics or draw anything persistent over the road.
- Don't show raw unsmoothed power, or re-derive smoothing differently per HUD.
- Don't let numerals reflow their neighbors (no proportional digits, no auto-sizing
  tiles).
- Don't animate for decoration: no pulsing values, no sliding panels mid-ride.
- Don't exceed ~4–5 panels or add a metric without removing/merging another.
- Don't color-code anything except power zones (and don't recolor labels per zone).
- Don't rely on blur or thin scrims (< 0.35 alpha) for legibility over bright scenes.
- Don't use box-drawing/ASCII gauges (`█░`) in shipped renders — draw real quads.
- Don't put buttons inside metric panels or metrics inside the control cluster.

## 7. Mapping onto the two implementations

### SwiftUI overlay (`shell-macos/Sources/VeloSim/UI/HUD/RideHUDOverlay.swift`)

Already close to doctrine. When touching it:
- Spacing/radii come from `Tok` (s1–s8, `rCard`, `rTile`, `glassGap`); type from
  `Typo` (§2 table); zone color from `PowerZone.color`. Never hardcode sizes/colors —
  if a needed token is missing, add it to `Tok`/`Typo`/`PowerZone`, don't inline.
- If aligning `PowerZone.color` to the canonical hexes (§4), define them once (asset
  catalog or `Color(hex:)` constants) so SwiftUI and any export path share values.
- Keep `.monospacedDigit()` + `.contentTransition(.numericText())` on live numerals;
  gate motion on `reduceMotion`, surfaces on `reduceTransparency`.
- New widgets (zone bar, W'bal gauge) belong inside `primaryCluster`'s glass container
  as additional tiles, not as new floating panels.

### Rust glyphon + quad HUD (`crates/velo-render/src/hud.rs`)

This HUD bakes into screenshots/eval renders and must *look like the same product*,
approximated with text runs + colored quads:
- **Panels as quads.** Replace the single text-list panel with one quad per region
  (§1): top strip, primary block, workout bar. Reuse `PanelUniform`/`write_panel` as
  an instanced or multi-draw quad pass. Panel fill: the existing dark
  `[0.03, 0.05, 0.08, 0.78]`; corner rounding is optional (an SDF fragment shader if
  cheap, square is acceptable).
- **Zone-colored power block.** Draw a quad behind (or a thick bar beside) the power
  numeral filled with the §4 zone hex at ~0.35–0.45 alpha over the dark panel, chosen
  from the same %FTP boundaries as `PowerZone.swift`. Numeral stays near-white
  `Color::rgb(235, 240, 245)`.
- **Type scale via multiple `Buffer`s.** One glyphon `Buffer`/`TextArea` per text
  role: hero power at ~3.5× the base metric size, secondary metrics at base, labels
  small/uppercase. A single 18 px monospace buffer cannot express the hierarchy.
- **Right-aligned numerals.** Measure each shaped run (`layout_runs()` gives
  `line_w`, as the current code already does) and set `TextArea.left` =
  `right_edge - line_w` so digits grow leftward without moving anything else. Pad
  values to fixed widths ("— " placeholders) to kill jitter between frames of a clip.
- **Workout bar as quads.** Replace the `█░` string gauge with two quads: track
  (white @ ~12% alpha) + fill (zone color of the *target* watts), with name/target/
  remaining as separate right/left-aligned runs above it.
- **Layout regions, not line count.** Position blocks by region anchors
  (margins from screen edges) instead of stacking `lines()`; keep the center clear in
  every aspect ratio velo-eval renders.
- Validate visually with `mcp__velo-eval__render_frame` / `hud_probe` after changes.

## Sources

- https://zwiftinsider.com/power-zone-colors/
- https://zwiftinsider.com/hud-refresh-closer-look/
- https://zwiftinsider.com/workout-hud/
- https://zracecentral.com/all-about-zwifts-heads-up-display/
- https://www.trainerroad.com/blog/cycling-power-zones-training-zones-explained/
- https://support.trainerroad.com/hc/en-us/articles/201974500-Live-Workout-Display-Explained
- https://support.wahoofitness.com/hc/en-us/articles/4402734188050-What-does-the-Power-Smoothing-function-do
- https://tempocyclist.com/2021/06/17/power-data-display-garmin-wahoo/
- https://www.garmin.com/en-GB/blog/coachs-corner-what-data-fields-to-have-on-your-garmin-edge-bike-computer/
- https://roadmancycling.com/blog/cycling-critical-power-w-prime-guide
- https://www.gamedeveloper.com/design/perceiving-without-looking-designing-huds-for-peripheral-vision
- https://gamedesignskills.com/game-design/racing/
- https://www.smashingmagazine.com/2023/08/designing-accessible-text-over-images-part1/
- https://www.wcag.com/blog/content-over-images-how-does-this-ux-ui-trend-impact-accessibility/
- https://gomakethings.com/preventing-layout-shift-with-numbers-using-css/
