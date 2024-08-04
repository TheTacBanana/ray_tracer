@group(0) @binding(0)
var<uniform> camera : Camera;

struct Camera {
    dimensions: vec2<f32>,
    fov: f32,
    pos: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.clip_position.x/camera.dimensions.x, in.clip_position.y/camera.dimensions.y, 0.0, 0.0);
}