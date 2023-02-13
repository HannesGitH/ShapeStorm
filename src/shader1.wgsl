
struct Primitive {
    position: vec3<f32>,
    rotation: vec4<f32>,
    data: vec4<f32>,
    instances: vec3<u32>,
    rgba: vec4<f32>,
    typus: u32,
    // operation: u32,
    // blend_strength: f32,
}

struct Primitives {
    prims: array<Primitive>,
}

@group(0) @binding(0)
var<storage, read> primitives: Primitives;

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    // @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // out.tex_coords = model.tex_coords;
    out.clip_position = 
    camera.view_proj * 
    vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
// @group(0) @binding(0)
// var t_diffuse: texture_2d<f32>;
// @group(0)@binding(1)
// var s_diffuse: sampler;

@fragment
fn fs_main(
    // @builtin(position) @invariant position: vec4<f32>,
    in: VertexOutput
    ) -> @location(0) vec4<f32> {
        // return vec4<f32>(normalize(in.clip_position.xyz), 1.0);
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // return vec4<f32>(in.color, 1.0);
    // return vec4<f32>(100.0,1.0,0.9, 1.0);
    return vec4<f32>(march(mk_ray_from_camera(in.clip_position.xy)).color.xyz, 1.0);
}

// ray marching

struct MarchOutput {
    distance: f32,
    color: vec4<f32>,
    steps: u32,
}

fn march(ray: Ray) -> MarchOutput {
    var dst = 100000.0;
    var steps = 0u;
    var color = vec4<f32>(.0);
    for (var i = 0u; i < 32u; i = i + 1u) {
        let out = calc_step(ray.origin + ray.direction * dst);
        dst = dst + out.distance;
        color = color + out.color/32.0;
        if (dst < 0.001) {
            steps = i;
            break;
        }
    }
    return MarchOutput(dst, color, steps);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn mk_ray_from_camera(uv: vec2<f32>) -> Ray {
    let origin = camera.view_proj[3].xyz;
    var direction = (camera.inverse_view_proj * vec4<f32>(uv,.0,1.0)).xyz;
    direction = (camera.view_proj * vec4<f32>(direction,.0)).xyz;
    direction = normalize(direction);
    return Ray(origin, direction);
}
fn distance_to_primitive(from_point: vec3<f32>, primitive: Primitive) -> f32 {
    var dst = 100000.0;
    let relative_point = from_point - primitive.position;
    dst = distance_to_box_frame(relative_point, primitive.data.xyz, primitive.data.w);
    // switch(primitive.typus) {
    //     case 0u: {dst = distance_to_box_frame(relative_point, primitive.data.xyz, primitive.data.w);}
    //     default: {}
    // }
    return dst;
}

struct StepOutput {
    distance: f32,
    color: vec4<f32>,
}

fn calc_step(from_point: vec3<f32>) -> StepOutput {
    var min_dst = 100000.0;
    // let pp = primitives.prims;
    // for (var i = 0; i < arrayLength(pp); i = i + 1) {
    //     let dst = distance_to_primitive(from_point, pp[i]);
    //     if (dst < min_dst) {
    //         min_dst = dst;
    //     }
    // }
    min_dst = distance_to_primitive(from_point, primitives.prims[0]);
    return StepOutput(min_dst, min_dst * primitives.prims[0].rgba);
}

fn quaternions_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32>{
	return v + 2.0*cross(q.xyz, cross(q.xyz,v) + q.w*v);
}

// primitive signed distance functions

fn distance_to_box_frame(from_point : vec3<f32>, box_size : vec3<f32>, frame_girth: f32)->f32
{
    let p:vec3<f32> = abs(from_point)-box_size;
    let q:vec3<f32> = abs(p+frame_girth)-frame_girth;
    return min(min(
        length(max(vec3(p.x,q.y,q.z),vec3(.0)))+min(max(p.x,max(q.y,q.z)),0.0),
        length(max(vec3(q.x,p.y,q.z),vec3(.0)))+min(max(q.x,max(p.y,q.z)),0.0)),
        length(max(vec3(q.x,q.y,p.z),vec3(.0)))+min(max(q.x,max(q.y,p.z)),0.0));
}