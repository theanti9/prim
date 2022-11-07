
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
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,

    @location(2) screen_pos: vec2<f32>,
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

    // out.clip_position = view_proj.view_proj * model_matrix * vec4<f32>(model.position, 1.0, 1.0);
    out.clip_position = vec4<f32>(model.position * 2.0, 1.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    out.screen_pos = (out.clip_position.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.screen_pos;
    let scene_color = textureSample(input_tex, input_sampler, uv);
    return vec4<f32>(in.screen_pos.x * scene_color.a, in.screen_pos.y * scene_color.a, 0.0, 1.0);
}