struct SceneUniforms {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> scene: SceneUniforms;

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
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.mvp * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.view_depth = out.clip_position.w;
    return out;
}

@group(1) @binding(0) var tile_tex: texture_2d<f32>;
@group(1) @binding(1) var tile_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(tile_tex, tile_sampler, input.uv);
    var col = base.rgb;
    // Shared atmosphere tail (game-graphics skill §4), same haze as
    // sky/terrain/scene so distance dissolves seamlessly.
    let fog = clamp(1.0 - exp(-input.view_depth / 900.0), 0.0, 0.88);
    let luma = dot(col, vec3<f32>(0.299, 0.587, 0.114));
    col = mix(col, vec3<f32>(luma), fog * 0.5);
    let haze = vec3<f32>(0.82, 0.87, 0.93);
    col = mix(col, haze, fog);
    col = (col - 0.5) * 1.06 + 0.51;
    let dith = fract(sin(dot(input.clip_position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    col += vec3<f32>((dith - 0.5) / 255.0);
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
