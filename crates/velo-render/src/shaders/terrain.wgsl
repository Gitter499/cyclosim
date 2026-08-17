// Textured terrain mesh pass.

struct Uniforms {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var terrain_tex: texture_2d<f32>;
@group(1) @binding(1) var terrain_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) view_depth: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    // Perspective w = distance along the view axis: free fog input.
    out.view_depth = out.clip_position.w;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(terrain_tex, terrain_sampler, in.uv);
    // Exponential distance fog into the sky's horizon haze.
    let fog = clamp(1.0 - exp(-in.view_depth / 900.0), 0.0, 0.88);
    let haze = vec3<f32>(0.82, 0.87, 0.93);
    return vec4<f32>(mix(base.rgb, haze, fog), base.a);
}
