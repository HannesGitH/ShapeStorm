
let zero_vec3 = vec3<f32>(0.0,0.0,0.0);

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = 
    camera.view_proj * 
    vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(
    // @builtin(position) @invariant position: vec4<f32>,
    in: VertexOutput
    ) -> @location(0) vec4<f32> {
        // return vec4<f32>(position.x,position.y,position.z, 1.0);
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // return vec4<f32>(in.color, 1.0);
}

fn box_frame_distance(from_point : vec3<f32>, box_size : vec3<f32>, frame_girth: f32)->f32
{
    let p:vec3<f32> = abs(from_point)-box_size;
    let q:vec3<f32> = abs(p+frame_girth)-frame_girth;
    return min(min(
        length(max(vec3(p.x,q.y,q.z),zero_vec3))+min(max(p.x,max(q.y,q.z)),0.0),
        length(max(vec3(q.x,p.y,q.z),zero_vec3))+min(max(q.x,max(p.y,q.z)),0.0)),
        length(max(vec3(q.x,q.y,p.z),zero_vec3))+min(max(q.x,max(q.y,p.z)),0.0));
}