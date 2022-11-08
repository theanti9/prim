
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

struct JumpFloodParams {
    level: f32,
    max_steps: f32,
    offset: f32,
}

@group(1) @binding(0)
var<uniform> jump_flood_params: JumpFloodParams;

@group(1) @binding(1)
var input_sampler: sampler;

@group(1) @binding(2)
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

    var closest_dist = 999999999.9;
    var closest_pos = vec2<f32>(0.0);

    let screen_pixel_size = vec2<f32>(1.0 / in.screen_size.x, 1.0 / in.screen_size.y);

    for (var x = -1.0; x <= 1.0; x = x + 1.0) {

        for (var y = -1.0; y <= 1.0; y = y + 1.0) {
            var voffset = in.screen_pos;
            voffset += vec2<f32>(x, y) * screen_pixel_size * jump_flood_params.offset;
            let pos = textureSample(input_tex, input_sampler, voffset).xy;
            let dist = distance(pos, in.screen_pos);
            // return vec4<f32>(pos, 0.0, 1.0);

            if pos.x != 0.0 && pos.y != 0.0 && dist < closest_dist {
                closest_dist = dist;
                closest_pos = pos;
            }
        }
    }

    return vec4<f32>(closest_pos, 0.0, 1.0);
}