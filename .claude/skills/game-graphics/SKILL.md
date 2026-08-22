---
name: game-graphics
description: Doctrine for VeloSim's stylized low-poly 3D scene look. Use for ANY work on the wgpu world rendering — lighting, color palette, sky, fog, scenery (trees/rocks/bushes), road, terrain, rider/bike models, shadows, post/atmosphere — i.e. the WGSL shaders in crates/velo-render/src/shaders and the meshes/vertex data feeding them. Not for the HUD (see hud-design).
---

# VeloSim Game Graphics Doctrine

Goal: a stylized low-poly outdoor world that reads as a *finished game* (Firewatch, Alto's
Adventure, Monument Valley, early Zwift) rather than programmer art — without PBR, texture
pipelines, or shadow mapping. The whole look is carried by four levers: **lighting, palette,
fog/atmosphere, silhouette**.

## §1 Art-direction rules

1. **One palette, few hues.** Pick 4-6 base hues for the whole scene and derive everything
   (lit/shadow variants, distance haze) from them. Harmonious restricted palettes are what make
   Alto's Adventure and Monument Valley read as designed, not decorated. (Source: Harry Nesbitt,
   "The Making of Alto's Adventure", harrynesbitt.com; Ken Wong, "The Art of Monument Valley",
   GDC 2015.)
2. **Value structure beats detail.** Separation comes from light/dark contrast, not texture.
   Ground plane mid-value, road slightly darker with light edge markings, scenery darker still,
   sky lightest. If a grayscale screenshot reads clearly, the color version will too.
3. **Warm light, cool shadow.** Sunlit faces shift warm (yellow), shadowed faces shift cool
   (blue), never toward gray/black. This single rule is most of the difference between "flat
   colored" and "lit". (Source: classic painting doctrine per James Gurney, *Color and Light*;
   applied in games in Mitchell et al., "Illustrative Rendering in Team Fortress 2", NPAR 2007.)
4. **Fog is art direction, not just distance culling.** Firewatch's look is largely layered
   atmospheric color: near objects saturated, far objects dissolve into a haze that belongs to
   the sky palette. Use fog to create depth planes deliberately. (Source: Jane Ng, "Making the
   World of Firewatch", GDC 2016.)
5. **Silhouette first.** Low-poly assets succeed or fail on outline. A tree should be
   identifiable as a black shape. Vary heights/widths per instance (deterministically seeded),
   avoid perfectly symmetric or axis-aligned shapes.
6. **What screams "programmer art"**: uniform unlit colors (no faceting), objects floating with
   no contact shadow, saturated pure-RGB primaries (e.g. `(0,1,0)` grass), identical repeated
   instances, banding in gradients, fog color that doesn't match sky. Fix these before adding
   anything new.

## §2 Lighting model

Single directional sun + hemispheric ambient, evaluated per vertex or per fragment. Exact recipe:

- **Sun direction** (world space, pointing *from* sun): normalize `(-0.45, -0.7, 0.35)` — high,
  slightly behind-left of the default chase camera so rider and scenery get a lit side facing
  the camera. Must be a uniform (shared across scene/terrain/tiles) so the whole world agrees.
- **Diffuse**: `ndl = max(dot(n, -sun_dir), 0.0)`. For rounded organic shapes you may use
  half-Lambert `ndl = pow(dot(n, -sun_dir) * 0.5 + 0.5, 2.0)` to keep shadow sides from going
  dead flat (Source: Valve, Half-Life shading; Mitchell et al. 2007). Plain Lambert is better
  for hard-faceted rocks/buildings.
- **Hemispheric ambient**: blend sky/ground by normal's up-ness:
  `ambient = mix(ground_amb, sky_amb, n.y * 0.5 + 0.5)` (Source: Tom Forsyth, "Hemisphere
  Lighting with Radiosity Maps", Game Developer / GDC 2003; same model as three.js
  HemisphereLight.)
- **Values** (normalized RGB, starting point — tune by screenshot):
  - `sun_color   = vec3(1.00, 0.95, 0.82)` × intensity `1.0` (warm)
  - `sky_amb     = vec3(0.45, 0.55, 0.70)` × `0.55` (cool, from zenith blue)
  - `ground_amb  = vec3(0.38, 0.36, 0.30)` × `0.55` (dull warm bounce)
  - `lit = albedo * (sun_color * ndl + mix(ground_amb, sky_amb, n.y*0.5+0.5))`, then clamp.
- **Normals for low-poly**: bake **per-face normals into duplicated vertices** (flat shading —
  the faceted look IS the style; Source: hextantstudios.com "Rendering Flat-Shaded / Low-Poly
  Style Models"). Do NOT compute normals via `dpdx/dpdy` cross products in the fragment shader:
  derivative precision is implementation-defined and threatens byte-stable renders. Smoothed
  normals only for deliberately soft shapes (rider body, rounded canopies).
- Billboard/crossed-quad assets have degenerate normals; either give them an authored fake
  normal (e.g. face-up hemisphere blend only) or keep them on a separate unlit-but-fogged path
  with hand-picked lit/shade colors baked into vertex color (see §3).

## §3 Scene element recipes

**Trees / bushes / rocks (crossed billboards today)**
- Two-tone canopy baked into vertex colors: top ~35% of canopy verts get the lit color
  (base × sun tint, ≈1.25× value), bottom gets shade color (base shifted cool, ≈0.65× value).
  This fakes top-down sun without any normals and works on crossed quads.
- Trunk darker and desaturated (bark ≈ `vec3(0.32, 0.26, 0.20)`).
- Deterministic per-instance variation: seed a hash from instance grid position; vary height
  ±20%, canopy hue ±0.03, and yaw. Never `rand()` from time.
- **Contact/blob shadow** under every object: a dark ellipse quad on the ground,
  `shadow_rgb = terrain color × 0.55`, alpha fading to 0 at the rim (radial gradient in vertex
  color or a tiny texture). Offset slightly along `-sun_dir` xz. Grounding via blob shadows is
  the classic cheap fix for "floating" objects and doubles as fake AO. (Source: 30fps.net,
  "Classic 3D videogame shadow techniques"; standard mobile practice per polycount.)

**Road**
- Edge contrast sells the road: asphalt mid-dark `≈ vec3(0.36, 0.36, 0.38)`, with a lighter
  shoulder line `≈ vec3(0.78, 0.77, 0.72)` and a darker gutter strip where road meets grass
  (fake AO seam, ≈0.7× asphalt). Cycling sims (Zwift, MyWhoosh) all keep the road corridor the
  highest-contrast band in the scene so the rider's path reads instantly.
- Center dashes: low-contrast (≈1.2× asphalt value), so they don't strobe at speed.
- Specular strip: optional cheap sheen — brighten asphalt where the view direction reflects
  toward the sun: `spec = pow(max(dot(reflect(view_dir, n), -sun_dir), 0.0), 32.0) * 0.15`,
  added as a neutral highlight. Only worth it once normals exist; keep subtle (wet-look kills
  the dry stylized read).

**Rider / bike**
- Readability at chase distance ≈ 8-12 m: the rider is the only *saturated accent* object.
  Give the jersey one high-chroma palette color (e.g. `vec3(0.85, 0.25, 0.20)`) not used
  anywhere in the environment; keep bike frame near-neutral dark. This is the Zwift/MyWhoosh
  pattern: muted world, vivid kits.
- Two-tone the rider like the canopy: top surfaces lit, undersides cool-shadowed; helmet
  lightest value on the model so the head silhouette reads.
- Blob shadow under both wheels (two small ellipses or one capsule) — grounding the rider is
  the single biggest believability win at chase distance.

**Sky**
- Keep gradient + sun disc; improve with: (a) a slightly warm horizon band tinted toward
  `sun_color` near the sun's azimuth, `mix(haze, sun_tint, pow(max(dot(view, sun_dir_to), 0.0), 8.0))`
  (Source: Inigo Quilez, "Fog", iquilezles.org/articles/fog — sun-tinted scattering); (b) make
  the disc world-anchored once the sky pass gets the camera basis, so sun position, sun_dir and
  fog tinting agree.
- Clouds: 2-4 flat, wide, hand-placed cloud shapes — either soft-edged billboards high in the
  sky or 2-3 overlapping `smoothstep` ellipses in the sky shader, colored
  `mix(white, haze, 0.3)`, flat bottoms, static (determinism). Avoid noise-based cloud fields —
  they read as mush at this style level.

## §4 Post / atmosphere

- **Fog ↔ sky coupling** (already correct — keep): haze `vec3(0.82, 0.87, 0.93)` is shared by
  sky horizon and all fogged passes; any change must touch all of them in the same PR. Cap fog
  at ~0.88 so distant geometry keeps a ghost of form (current behavior; matches Firewatch's
  "objects dissolve but never vanish").
- **Distance desaturation**: before mixing to haze, pull distant colors toward their own luma:
  `let luma = dot(col, vec3(0.299, 0.587, 0.114)); col = mix(col, vec3(luma), fog * 0.5);`
  This gives aerial perspective (near = saturated, far = gray-blue) beyond what haze-mix alone
  does. (Source: aerial-perspective doctrine per Gurney; Firewatch GDC 2016.)
- **Tone curve**: a gentle filmic-ish curve lifts the look from "raw output":
  `col = col * (1.0 + col * 0.15); col = pow(clamp(col, vec3(0.0), vec3(1.0)), vec3(1.0/1.05));`
  or simply a contrast pivot `col = (col - 0.5) * 1.06 + 0.5 + 0.01`. Keep it in each fragment
  shader's tail (no post pass exists); factor into a shared WGSL snippet pasted per shader.
- **Dithering**: smooth gradients (sky, fog) band on 8-bit targets. Add screen-space hash noise
  before writeout: `col += (hash(pixel_xy) - 0.5) / 255.0;` with
  `fn hash(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }`
  Position-hash only — deterministic per pixel, byte-stable across runs on the same backend.
  "There's no reason a game should ship with banding." (Source: Mikkel Gjøl, Playdead, "Banding
  in Games: A Noisy Rant", loopit.dk/banding_in_games.pdf; and "Low Complexity, High Fidelity:
  The Rendering of INSIDE", GDC 2016.)

## §5 Implementation notes for THIS codebase

- Shaders: `crates/velo-render/src/shaders/{scene,terrain,tiles,sky,bike}.wgsl`. `scene.wgsl`
  and `bike.wgsl` are the colored-vertex pipelines (position+RGB, no normals); `terrain.wgsl` /
  `tiles.wgsl` are textured; `sky.wgsl` is a fullscreen triangle with no camera uniforms yet.
- **Adding normals**: extend the colored-vertex layout to `position + color + normal`
  (`@location(2) normal: vec3<f32>`), update the Rust vertex struct / `VertexBufferLayout`
  stride and the mesh builders that emit crossed quads. Duplicate vertices per face for flat
  shading. Lighting params (sun_dir, sun/sky/ground colors) belong in the existing uniform
  struct — mind WGSL uniform alignment (`vec3<f32>` aligns to 16 bytes; pad or use `vec4`).
- **Determinism**: renders are byte-compared in tests. No time uniforms in stills, no
  `dpdx/dpdy`-derived shading, no non-seeded randomness in mesh generation; dither/hash from
  pixel position only. Expect golden images to change on any look PR — regenerate intentionally,
  one visual change per PR so diffs are reviewable.
- **Portability (Metal via naga + Vulkan/lavapipe)**: don't use WGSL reserved identifiers as
  names — `half`, `filter`, `sampler`, `texture`, `mat`, `premerge`, etc. (`half` is the classic
  Metal-clash; it is on the WGSL reserved-word list — Source: W3C WGSL spec §keywords). Avoid
  `f16`. Watch `clamp`/`pow` on possibly-negative bases (`pow(x, y)` is undefined for `x < 0` —
  clamp first). lavapipe (CPU) output is the CI truth; verify screenshots via
  `mcp__velo-eval__render_frame` after every shader change.
- Fog currently keys off `clip_position.w` with density `1/900` — reuse that exact input for
  desaturation/tinting so all passes stay in lockstep.

## §6 Ranked iteration plan (impact per effort, one PR each)

1. **Blob shadows** under trees/rocks/rider (ground-tinted ellipses, existing unlit pipeline —
   no shader changes). Biggest "it's a real place now" win.
2. **Two-tone vertex-color pass over existing assets**: lit tops / cool-shadowed bottoms on
   canopies + rider, warm/cool per §1.3, kill any pure-RGB primaries. Pure data change.
3. **Normals + sun/hemisphere lighting** in `scene.wgsl` (§2) for terrain-adjacent meshes and
   any true 3D scenery; migrate `bike.wgsl` too (and add its missing fog while there).
4. **Dithering + distance desaturation + shared tone curve** in all fragment shaders (§4).
5. **Road dressing**: gutter AO seam, brighter shoulder lines, low-contrast center dashes
   (terrain texture / tile content change).
6. **Sky upgrade**: camera-aware sun position, sun-tinted horizon, 2-3 static clouds (§3 Sky).
7. **Rider readability**: accent-color jersey, helmet value pop, wheel blob shadows (§3 Rider).
8. **Deterministic per-instance scenery variation** (seeded height/hue/yaw) to break repetition.
9. **Road specular strip** (needs normals from step 3; keep at 0.15 strength).

After each step: render the standard eval frames, check grayscale readability (§1.2), check the
horizon seam, and confirm CI byte-stability before regenerating goldens.
