// Textured terrain pass: vertex-normal sun/hemisphere lighting + atmosphere
// (game-graphics skill §2/§4).

struct Uniforms {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var terrain_tex: texture_2d<f32>;
@group(1) @binding(1) var terrain_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) view_depth: f32,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    // Perspective w = distance along the view axis: free fog input.
    out.view_depth = out.clip_position.w;
    out.normal = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(terrain_tex, terrain_sampler, in.uv);
    let n = normalize(in.normal);

    // Warm sun + cool-sky/warm-ground hemisphere ambient (skill §2).
    let to_sun = normalize(vec3<f32>(0.45, 0.70, -0.35));
    let sun_color = vec3<f32>(1.00, 0.95, 0.82);
    let sky_amb = vec3<f32>(0.45, 0.55, 0.70) * 0.55;
    let ground_amb = vec3<f32>(0.38, 0.36, 0.30) * 0.55;
    let ndl = max(dot(n, to_sun), 0.0);
    let ambient = mix(ground_amb, sky_amb, n.y * 0.5 + 0.5);
    var col = base.rgb * clamp(sun_color * ndl + ambient, vec3<f32>(0.0), vec3<f32>(1.4));

    // Atmosphere (skill §4): distance desaturation, haze mix, tone, dither.
    let fog = clamp(1.0 - exp(-in.view_depth / 900.0), 0.0, 0.88);
    let luma = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = mix(col, vec3<f32>(luma), fog * 0.5);
    let haze = vec3<f32>(0.82, 0.87, 0.93);
    col = mix(col, haze, fog);
    col = (col - 0.5) * 1.06 + 0.51;
    let dith = fract(sin(dot(in.clip_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    col += vec3<f32>((dith - 0.5) / 255.0);
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), base.a);
}
