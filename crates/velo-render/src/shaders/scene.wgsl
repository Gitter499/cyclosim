// Colored-vertex pass for billboarded scenery + fallback grid: unlit (lit and
// shade tones are baked into vertex colors — game-graphics skill §2/§3), with
// the shared atmosphere tail (§4).

struct Uniforms {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) view_depth: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.view_depth = out.clip_position.w;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var col = in.color;
    // Atmosphere (skill §4): desaturate with distance, fade into the sky
    // haze, gentle contrast, position-hash dither against banding.
    let fog = clamp(1.0 - exp(-in.view_depth / 900.0), 0.0, 0.88);
    let luma = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = mix(col, vec3<f32>(luma), fog * 0.5);
    let haze = vec3<f32>(0.82, 0.87, 0.93);
    col = mix(col, haze, fog);
    col = (col - 0.5) * 1.06 + 0.51;
    let dith = fract(sin(dot(in.clip_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    col += vec3<f32>((dith - 0.5) / 255.0);
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
