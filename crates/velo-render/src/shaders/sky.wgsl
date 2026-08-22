// Fullscreen gradient sky: haze at the horizon rising to a deeper zenith blue.
// Drawn first with depth writes off; scene geometry overdraws it. The horizon
// color must match the fog haze in terrain/tiles/scene shaders so distant
// geometry dissolves into the sky seamlessly.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Single triangle covering the screen.
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos[vi], 1.0, 1.0);
    out.ndc = pos[vi];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = clamp(in.ndc.y * 0.5 + 0.5, 0.0, 1.0);
    let horizon = vec3<f32>(0.82, 0.87, 0.93);
    let zenith = vec3<f32>(0.33, 0.55, 0.83);
    // Ease toward haze near the horizon for a soft atmospheric band.
    let k = pow(t, 1.5);
    var col = mix(horizon, zenith, k);

    // Sun-tinted scattering: the horizon warms toward the sun's side
    // (game-graphics skill §3 Sky, after Inigo Quilez's fog article).
    let sun_side = clamp(1.0 - abs(in.ndc.x + 0.38) * 0.7, 0.0, 1.0);
    let warm = pow(sun_side, 3.0) * (1.0 - t) * 0.30;
    col = mix(col, vec3<f32>(0.98, 0.90, 0.78), warm);

    // Soft sun disc + wide glow, high left of center. Screen-anchored (this
    // pass has no camera uniforms) — placeholder-tier skybox; the glow stays
    // well above the horizon so the fog-haze seam is untouched. The x scale
    // approximates a 16:9 aspect so the disc reads round.
    let sun_pos = vec2<f32>(-0.38, 0.62);
    let dv = (in.ndc - sun_pos) * vec2<f32>(1.72, 1.0);
    let d = length(dv);
    let disc = smoothstep(0.075, 0.045, d);
    let glow = smoothstep(0.55, 0.0, d) * 0.16;
    let sun_color = vec3<f32>(1.0, 0.97, 0.86);
    col = mix(col, sun_color, clamp(disc + glow, 0.0, 1.0));

    // Soft cumulus band: two octaves of value noise, flattened vertically so
    // clouds read as distant stratus. Static (deterministic renders).
    let p = vec2<f32>(in.ndc.x * 1.72, in.ndc.y) * vec2<f32>(2.2, 6.0);
    let n = vnoise(p) * 0.62 + vnoise(p * 2.7 + vec2<f32>(13.1, 7.7)) * 0.38;
    // Only in the upper sky, denser toward the top, never over the horizon.
    let band = smoothstep(0.12, 0.55, in.ndc.y);
    let cloud = smoothstep(0.58, 0.78, n) * band * 0.75;
    col = mix(col, vec3<f32>(0.99, 0.99, 1.0), cloud);
    return vec4<f32>(col, 1.0);
}

// Value noise: hash-based, no time input — renders stay byte-stable.
fn hash2(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
