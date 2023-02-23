
struct Primitive {
    position: vec3<f32>,
    _speed: f32,
    rotation: vec4<f32>,
    _rotation_delta: vec4<f32>,
    data: vec4<f32>,
    instances: vec3<u32>,
    instances_distance: f32,
    rgba: vec4<f32>,
    typus: u32,
    twist: f32,
    // operation: u32,
    // blend_strength: f32,
}

struct Primitives {
    // length: u32,
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
    effect: u32,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

// Vertex shader
struct VertexInput {
    // @location(0) position: vec3<f32>,
    @builtin(vertex_index) v_idx: u32
    // @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // @location(0) tex_coords: vec2<f32>,
}

var<private> v_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
);


var<private> indices: array<u32,6> = array<u32, 6>(2u, 1u, 0u, 3u, 2u, 0u);

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // out.tex_coords = model.tex_coords;
    out.clip_position = 
    // camera.world_to_screen * 
    vec4<f32>(v_positions[indices[model.v_idx]], 0.0, 1.0);
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
    // return vec4<f32>(1.0);
}

// ray marching

struct MarchOutput {
    distance: f32,
    color: vec4<f32>,
    steps: u32,
}

const max_steps = 32u;
const max_distance = 1000.0;
const epsilon = 1.0;
    
fn march(ray: Ray) -> MarchOutput {
    var dst = 0.0;
    var steps = 0u;
    let color_damper = f32(max_steps)/6.0;
    // let max_steps_f32_x3 = max_steps_f32;
    var color = vec4<f32>(.0);
    for (var i = 0u; i < max_steps; i = i + 1u) {
        let out = calc_step(ray.origin + ray.direction * dst);
        dst = dst + out.distance;
        if (out.distance < epsilon) {
            steps = i;
            // color = out.color;
            if (camera.effect == 4u) { //4u = black-body
                color = vec4<f32>(0.0);
            } else if (camera.effect == 5u) { //3u = white-body
                color = vec4<f32>(1.0);
            } else { //shattered glass looking default shader
                color = color * (max_distance - dst) / max_distance;
            }
            break;
        }
        if (camera.effect != 2u) { //2u = glow-off
            if (camera.effect == 5u) {
                color = color + out.color / color_damper;
            } else {
                color = color + out.color * (max_distance - dst) / max_distance / color_damper;
            }
            // color = color 
        }
        if (dst > max_distance) {
            steps = i;
            if (camera.effect == 1u) { //1u = glassy-onion
                color = out.color;
            }
            break;
        }
    }
    // color = color * (max_distance - dst) / max_distance;
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
    let infinite_repition_period = vec2<f32>(1000.0,1000.0);
    //translate to primitive space
    var q = from_point-primitive.position;
    //infinite repition
    //// let relative_point_q = (modf(q/c+0.5).fract-vec3<f32>(0.5))*c; //to spec
    var whole = vec2<f32>();
    let mod_point = vec3<f32>((modf(q.xy/infinite_repition_period+0.5*infinite_repition_period,&whole)-vec2<f32>(0.5))*infinite_repition_period,q.z); //to old spec
    let relative_point = fast_inverse_qrotate_vector(primitive.rotation,mod_point);// - fast_inverse_qrotate_vector(primitive.rotation,primitive.position); 
    q = relative_point;
    //// let relative_point = qrotate_vector(qinverse(primitive.rotation),from_point) - qrotate_vector(qinverse(primitive.rotation),primitive.position); 
    // // twisting //FIXME: this is not working, it brings enourmous amounts of noise
    // let twist = primitive.twist;
    // let cos_twist = cos(twist*q.y);
    // let sin_twist = sin(twist*q.y);
    // let twist_matrix = mat2x2(cos_twist,-sin_twist,sin_twist,cos_twist);
    // let twisted_point = vec3(twist_matrix*q.xz,q.y);
    // q = twisted_point;
    // finite instancing
    let dis : vec3<f32> = round(q/primitive.instances_distance);
    let bound = vec3<f32>(primitive.instances);
    let instanced_point : vec3<f32> = q-primitive.instances_distance*clamp(dis,-bound,bound);
    //// dst = distance_to_box_frame(relative_point_q, primitive.data);
    q = instanced_point;
    switch(primitive.typus) {
        case 0u: {dst = distance_to_box_frame(q, primitive.data);}
        case 1u: {dst = distance_to_ellipsoid(q, primitive.data);}
        case 2u: {dst = distance_to_octahedron(q, primitive.data);}
        case 3u: {dst = distance_to_chain_link(q, primitive.data);}
        default: {}
    }
    return dst;
}

struct StepOutput {
    distance: f32,
    color: vec4<f32>,
}

fn calc_step(from_point: vec3<f32>) -> StepOutput {
    var min_dst = 100000.0;
    var color = vec4<f32>(0.0);
    for (var i:u32 = 0u; i < arrayLength(&primitives.prims); i = i + 1u) {
        let prim = get_ith_primitive(i);
        let dst = distance_to_primitive(from_point, prim);
        if (camera.effect == 3u) {//clean-from-water
            color = color + prim.rgba / max(dst*dst*dst/max_distance,1.0);
        } else {
            color = color + prim.rgba / max(dst/3.0,1.0);
        }
        if (dst < min_dst) {
            min_dst = dst;
        }
    }
    return StepOutput(min_dst, color);
}

fn get_ith_primitive(i: u32) -> Primitive {
    return primitives.prims[i];
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

fn distance_to_ellipsoid(from_point: vec3<f32>, sphere_data: vec4<f32>) -> f32 {
    // thats would be a sphere
    // let sphere_radius = sphere_data.w;
    // return length(from_point) - sphere_radius;
    // degree two approximation
    let k0 : f32 = length(from_point/sphere_data.xyz);
    let k1 : f32 = length(from_point/sphere_data.xyz/sphere_data.xyz);
    return k0 * ( k0 - 1.0 ) / k1;
}

const sqrt_third = 0.57735026918962576450914878050196; //thanks copilot

fn distance_to_octahedron(from_point: vec3<f32>, octa_data: vec4<f32>) -> f32 {
    let octa_size = octa_data.x;
    let p = abs(from_point);
    return (p.x+p.y+p.z-octa_size)*sqrt_third;
}

fn distance_to_chain_link(from_point: vec3<f32>, chain_data: vec4<f32>) -> f32 {
    let len = chain_data.x;
    let arc_radius = chain_data.y;
    let girth = chain_data.z;
    let q = vec3( from_point.x, max(abs(from_point.y)-len,0.0), from_point.z );
    return length(vec2(length(q.xy)-arc_radius,q.z)) - girth;
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