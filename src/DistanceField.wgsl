struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
    @location(10) emitter_occluder: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    screen_width: u32,
    screen_height: u32,
}

@group(0) @binding(0)
var<uniform> view_proj: CameraUniform;

@group(1) @binding(0)
var input_sampler: sampler;

@group(1) @binding(1)
var input_tex: texture_2d<f32>;


struct VertexInput {
    @location(0) position: vec2<f32>,
    @builtin(vertex_index) vertex_index: u32,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(2) screen_pos: vec2<f32>,
    @location(3) screen_size: vec2<f32>,
}


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;

    out.clip_position = vec4<f32>(model.position * 2.0, 1.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    out.screen_pos = (out.clip_position.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5));
    out.screen_size = vec2<f32>(f32(view_proj.screen_width), f32(view_proj.screen_height));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(input_tex, input_sampler, in.screen_pos);
    let dist = distance(tex.xy, in.screen_pos);
    // Controls lighting distance.
    let dist_mod = 0.205;
    let mapped = clamp(dist * dist_mod, 0.0, 1.0);
    return vec4<f32>(vec3<f32>(mapped), 1.0);
}