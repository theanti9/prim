
let PI = 3.141596;
let rays_per_pixel = 32;
let emission_multiplier = 1.0;
let max_raymarch_steps = 32;
let dist_mod = 1.0;

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
var distance_tex: texture_2d<f32>;

@group(1) @binding(2)
var emitter_occluder_tex: texture_2d<f32>;

@group(1) @binding(3)
var<uniform> time: f32;


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

fn random(st: vec2<f32>) -> f32 {
    return fract(sin(dot(st.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453123);
}

struct HitResult {
    hit_pos: vec2<f32>,
    hit: bool,
}

fn raymarch(origin: vec2<f32>, dir: vec2<f32>, aspect: f32) -> HitResult {
    var out: HitResult;

    var current_dist = 0.0;
    for (var i = 0; i < max_raymarch_steps; i++) {
        var sample_point = origin + dir * current_dist;
        sample_point.x /= aspect;

        if sample_point.x > 1.0 || sample_point.x < 0.0 || sample_point.y > 1.0 || sample_point.y < 0.0 {
            break;
        }

        let dist_to_surface = textureSample(distance_tex, input_sampler, sample_point).r / dist_mod;

        if dist_to_surface < 0.001 {
            out.hit_pos = sample_point;
            out.hit = true;
            break;
        }

        current_dist += dist_to_surface;
    }

    return out;
}

struct EmissiveData {
    emissive: f32,
    color: vec3<f32>,
}

fn get_surface(uv: vec2<f32>) -> EmissiveData {
    var emissive_data: EmissiveData;

    let sampled_data = textureSample(emitter_occluder_tex, input_sampler, uv);
    emissive_data.emissive = max(sampled_data.r, max(sampled_data.g, sampled_data.b)) * emission_multiplier;
    emissive_data.color = sampled_data.rgb;

    return emissive_data;
}

fn lin_to_srgb(color: vec4<f32>) -> vec3<f32> {
    let x = color.rgb * 12.92;
    let y = 1.055 * pow(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(0.4166667)) - 0.055;
    var clr = color.rgb;

    if color.r < 0.00313308 { clr.r = x.r; } else { clr.r = y.r; }
    if color.g < 0.00313308 { clr.g = x.g; } else { clr.g = y.g; }
    if color.b < 0.00313308 { clr.b = x.b; } else { clr.b = y.b; }
    return clr.rgb;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixel_emis = 0.0;
    var pixel_col = vec3<f32>(0.0);

    let aspect = in.screen_size.y / in.screen_size.x;
    var uv = in.screen_pos;
    uv.x *= aspect;

    // This is super expensive. Sample noise texture instead?
    let rand2pi = random(in.screen_pos * vec2<f32>(time, -time)) * 2.0 * PI;
    let golden_angle = PI * 0.7639320225; // magic number that gives us good ray distribution

    for (var i = 0; i < rays_per_pixel; i++) {
        let cur_angle = rand2pi + golden_angle * f32(i);
        // let cur_angle = golden_angle * f32(i);
        let ray_dir = normalize(vec2<f32>(cos(cur_angle), sin(cur_angle)));
        let ray_origin = uv;
        let hit_result = raymarch(ray_origin, ray_dir, aspect);
        if hit_result.hit {
            let emissive_data = get_surface(hit_result.hit_pos);

            pixel_emis += emissive_data.emissive;
            pixel_col += emissive_data.color;
        }
    }

    pixel_col = pixel_col / pixel_emis;
    pixel_emis /= f32(rays_per_pixel);

    return vec4<f32>(lin_to_srgb(vec4<f32>(pixel_emis * pixel_col, 1.0)), 1.0);
}