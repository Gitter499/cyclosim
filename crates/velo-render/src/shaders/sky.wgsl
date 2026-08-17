// Fullscreen gradient sky: haze at the horizon rising to a deeper zenith blue.
// Drawn first with depth writes off; scene geometry overdraws it. The horizon
// color must match the fog haze in terrain/tiles/scene shaders so distant
// geometry dissolves into the sky seamlessly.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc_y: f32,
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
    out.ndc_y = pos[vi].y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = clamp(in.ndc_y * 0.5 + 0.5, 0.0, 1.0);
    let horizon = vec3<f32>(0.82, 0.87, 0.93);
    let zenith = vec3<f32>(0.33, 0.55, 0.83);
    // Ease toward haze near the horizon for a soft atmospheric band.
    let k = pow(t, 1.5);
    return vec4<f32>(mix(horizon, zenith, k), 1.0);
}
