
struct Primitive {
    position: vec3<f32>,
    rotation: vec4<f32>,
    data: vec4<f32>,
    instances: vec3<u32>,
    instances_distance: f32,
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
    view_position: vec4<f32>,
    // view_proj: mat4x4<f32>,
    // inverse_proj: mat4x4<f32>,
    // cam_to_world: mat4x4<f32>,
    world_to_screen: mat4x4<f32>,
    screen_to_world: mat4x4<f32>,
    pixel_normalization: mat4x4<f32>,
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
    // camera.world_to_screen * 
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
        // return vec4<f32>(normalize(in.clip_position.xyz/1000.0), 1.0);
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // return vec4<f32>(in.color, 1.0);
    // return vec4<f32>(100.0,1.0,0.9, 1.0);
    let ray = mk_ray_from_camera((camera.pixel_normalization * in.clip_position).xy/*-vec2<f32>(500.0,500.0)*/);
    // return vec4<f32>(ray.direction,1.0);
    let out = march(ray);
    // // return vec4<f32>(f32(out.steps)/32.0, vec3<f32>(1.0));
    return vec4<f32>(out.color.xyz , 1.0);
}

// ray marching

struct MarchOutput {
    distance: f32,
    color: vec4<f32>,
    steps: u32,
}

const max_steps = 16u;
const max_distance = 1000.0;
const epsilon = 1.0;
    
fn march(ray: Ray) -> MarchOutput {
    var dst = 0.0;
    var steps = 0u;
    let max_steps_f32 = f32(max_steps);
    // let max_steps_f32_x3 = max_steps_f32;
    var color = vec4<f32>(.0);
    for (var i = 0u; i < max_steps; i = i + 1u) {
        let out = calc_step(ray.origin + ray.direction * dst);
        dst = dst + out.distance;
        color = color + out.color;
        if (out.distance < epsilon) {
            steps = i;
            // color = out.color;
            color = vec4<f32>(1.0);
            break;
        }
        // if (dst > max_distance) {
        //     steps = i;
        //     color = vec4<f32>(0.0);
        //     break;
        // }
    }
    return MarchOutput(dst, color, steps);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn mk_ray_from_camera(uv: vec2<f32>) -> Ray {
    let origin = camera.view_position.xyz;
    var direction = (camera.screen_to_world * vec4<f32>(uv,0.0,1.0)).xyz;
    direction = normalize(direction-origin);
    return Ray(origin, direction);
}
fn distance_to_primitive(from_point: vec3<f32>, primitive: Primitive) -> f32 {
    var dst = 100000.0;
    let c = vec3<f32>(1000.0,1000.0,1000.0);
    let q = from_point;
    // let relative_point_q = (modf(q/c+0.5).fract-vec3<f32>(0.5))*c; //to spec
    var whole = vec3<f32>();
    let mod_point = (modf(q/c+0.5*c,&whole)-vec3<f32>(0.5))*c; //to old spec
    let relative_point = fast_inverse_qrotate_vector(primitive.rotation,mod_point) - fast_inverse_qrotate_vector(primitive.rotation,primitive.position); 
    // let relative_point = qrotate_vector(qinverse(primitive.rotation),from_point) - qrotate_vector(qinverse(primitive.rotation),primitive.position); 
    // let dis : vec3<f32> = round(relative_point/primitive.instances_distance);
    // let bound = vec3<f32>(primitive.instances);
    // let q : vec3<f32> = relative_point-primitive.instances_distance*clamp(dis,-bound,bound);
    // let relative_point_q = fast_inverse_qrotate_vector(primitive.rotation,q) - fast_inverse_qrotate_vector(primitive.rotation,primitive.position); //we can rotate the multiple instances in itself
    dst = distance_to_box_frame(relative_point, primitive.data);
    // switch(primitive.typus) {
    //     case 0u: {dst = distance_to_box_frame(relative_point, primitive.data);}
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
    return StepOutput(min_dst, primitives.prims[0].rgba / min_dst);
}


// primitive signed distance functions

fn distance_to_box_frame(from_point : vec3<f32>, box_data : vec4<f32>)->f32
{
    let box_size:vec3<f32> = box_data.xyz;
    let frame_girth: f32 = box_data.w;
    let p:vec3<f32> = abs(from_point)-box_size;
    let q:vec3<f32> = abs(p+frame_girth)-frame_girth;
    return min(min(
        length(max(vec3(p.x,q.y,q.z),vec3(.0)))+min(max(p.x,max(q.y,q.z)),0.0),
        length(max(vec3(q.x,p.y,q.z),vec3(.0)))+min(max(q.x,max(p.y,q.z)),0.0)),
        length(max(vec3(q.x,q.y,p.z),vec3(.0)))+min(max(q.x,max(q.y,p.z)),0.0));
}

fn distance_to_sphere(from_point: vec3<f32>, sphere_data: vec4<f32>) -> f32 {
    let sphere_radius = sphere_data.w;
    return length(from_point) - sphere_radius;
}


// quaternions
fn qmul(q1: vec4<f32>,  q2:vec4<f32>)->vec4<f32>
{
    return vec4<f32>(
        q2.xyz * q1.w + q1.xyz * q2.w + cross(q1.xyz, q2.xyz),
        q1.w * q2.w - dot(q1.xyz, q2.xyz)
    );
}

// fn qrotate_vector(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
//     return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
// }

fn qconj(q: vec4<f32>) -> vec4<f32> {
    return q * vec4<f32>(-1.0, -1.0, -1.0, 1.0);
}

fn qinverse(q: vec4<f32>) -> vec4<f32> {
    return qconj(q) / dot(q, q);
}

fn qrotate_vector( r:vec4<f32>, v:vec3<f32>)->vec3<f32>
{
    return qmul(r, qmul(vec4<f32>(v, .0), qconj(r))).xyz;
}

fn fast_inverse_qrotate_vector( r:vec4<f32>, v:vec3<f32>)->vec3<f32>
{
    let rr = r / dot(r, r);
    let rhs = vec4<f32>(v * rr.w + cross(v, rr.xyz), - dot(v, rr.xyz));
    return rhs.xyz * r.w - r.xyz * rhs.w - cross(r.xyz, rhs.xyz);
}