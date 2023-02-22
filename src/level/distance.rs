use cgmath::{
    dot, num_traits::clamp, BaseFloat, ElementWise, InnerSpace, Quaternion, Rad, Rotation, Vector2,
    Vector3, Vector4,
};
use winit::dpi::Pixel;

use crate::primitives::{SDFPrimitive, Typus};

fn distance_to_primitive(from_point: Vector3<f32>, primitive: SDFPrimitive) -> f32 {
    let infinite_repition_period = Vector2::new(1000.0, 1000.0);
    //translate to primitive space
    let mut q: Vector3<f32> =
        from_point - <[f32; 3] as Into<Vector3<f32>>>::into(primitive.position);
    //infinite repetition
    let (x, y) = (
        (q.x % infinite_repition_period.x + infinite_repition_period.x)
            % infinite_repition_period.x,
        (q.y % infinite_repition_period.y + infinite_repition_period.y)
            % infinite_repition_period.y,
    );
    let mod_point = Vector3::new(x, y, q.z);
    let relative_point = fast_inverse_qrotate_vector(primitive.rotation, mod_point);
    q = relative_point;
    //// let relative_point = qrotate_vector(qinverse(primitive.rotation),from_point) - qrotate_vector(qinverse(primitive.rotation),primitive.position);
    // // twisting //FIXME: this is not working, it brings enourmous amounts of noise
    // let twist = primitive.twist;
    // let cos_twist = cos(twist*q.y);
    // let sin_twist = sin(twist*q.y);
    // let twist_matrix = mat2x2(cos_twist,-sin_twist,sin_twist,cos_twist);
    // let twisted_point = Vector3(twist_matrix*q.xz,q.y);
    // q = twisted_point;
    // finite instancing
    let dis = q
        .div_element_wise(vec3_from(primitive.instances_distance))
        .round_element_wise();
    let bound: Vector3<f32> = Vector3::new(
        primitive.instances[0].cast(),
        primitive.instances[1].cast(),
        primitive.instances[2].cast(),
    );
    let instanced_point: Vector3<f32> =
        q - primitive.instances_distance * clamp_element_wise(dis, zero_vec3 - bound, bound);
    //// dst = distance_to_box_frame(relative_point_q, primitive.data);
    q = instanced_point;
    match primitive.typus {
        Typus::BoxFrame => distance_to_box_frame(q, primitive.data),
        Typus::Ellipsoid => distance_to_ellipsoid(q, primitive.data),
        Typus::Octahedron => distance_to_octahedron(q, primitive.data),
        Typus::ChainLink => distance_to_chain_link(q, primitive.data),
        _ => 100000.0,
    }
}

const zero_vec3: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
fn distance_to_box_frame(from_point: Vector3<f32>, box_data: [f32; 4]) -> f32 {
    let box_size = Vector3::new(box_data[0], box_data[1], box_data[2]);
    let frame_girth = vec3_from(box_data[3]);
    let p = abs(from_point) - box_size;
    let q: Vector3<f32> = abs(p + frame_girth) - frame_girth;
    return Vector3::new(p.x, q.y, q.z)
        .max_element_wise(zero_vec3)
        .magnitude()
        + p.x
            .max(q.y.max(q.z))
            .min(0.0)
            .min(
                Vector3::new(q.x, p.y, q.z)
                    .max_element_wise(zero_vec3)
                    .magnitude()
                    + q.x.max(p.y.max(q.z)).min(0.0),
            )
            .min(
                Vector3::new(q.x, q.y, p.z)
                    .max_element_wise(zero_vec3)
                    .magnitude()
                    + q.x.max(q.y.max(p.z)).min(0.0),
            );
}

fn distance_to_ellipsoid(from_point: Vector3<f32>, sphere_data: [f32; 4]) -> f32 {
    // thats would be a sphere
    // let sphere_radius = sphere_data.w;
    // return length(from_point) - sphere_radius;
    // degree two approximation
    let xyz = Vector3::<f32>::new(sphere_data[0], sphere_data[1], sphere_data[2]);
    let k0: f32 = from_point.div_element_wise(xyz).magnitude();
    let k1: f32 = from_point
        .div_element_wise(xyz)
        .div_element_wise(xyz)
        .magnitude();
    return k0 * (k0 - 1.0) / k1;
}

const sqrt_third: f32 = 0.57735026918962576450914878050196; //thanks copilot
fn distance_to_octahedron(from_point: Vector3<f32>, octa_data: [f32; 4]) -> f32 {
    let octa_size = octa_data[0];
    let p: Vector3<f32> = abs(from_point);
    return (p.x + p.y + p.z - octa_size) * sqrt_third;
}

fn distance_to_chain_link(from_point: Vector3<f32>, chain_data: [f32; 4]) -> f32 {
    let len = chain_data[0];
    let arc_radius = chain_data[1];
    let girth = chain_data[2];
    let q = Vector3::new(
        from_point.x,
        (from_point.y.abs() - len).max(0.0),
        from_point.z,
    );
    return Vector2::new(Vector2::new(q.x, q.y).magnitude() - arc_radius, q.z).magnitude() - girth;
}

fn fast_inverse_qrotate_vector(r: [f32; 4], v: Vector3<f32>) -> Vector3<f32> {
    // let rr = r / dot(r, r);
    // let xyz = v * rr.w + v.cross(rr.xyz);
    // let rhs = [xyz.x,xyz.y,xyz.z, - dot(v, rr.xyz)];
    // return rhs[..3] * r[3] - r.xyz * rhs.w - r.xyz.cross( rhs.xyz);
    let qr: Quaternion<f32> = r.into();
    qr.invert() * v
}

fn abs(v: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(v.x.abs(), v.y.abs(), v.z.abs())
}

trait ElementWisePro<S: BaseFloat> {
    fn max_element_wise(self, other: Self) -> Self;
    fn round_element_wise(self) -> Self;
}

impl ElementWisePro<f32> for Vector3<f32> {
    fn max_element_wise(self, other: Self) -> Self {
        Vector3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }
    fn round_element_wise(self) -> Self {
        Vector3::new(self.x.round(), self.y.round(), self.z.round())
    }
}

fn vec3_from(f: f32) -> Vector3<f32> {
    Vector3::new(f, f, f)
}
fn clamp_element_wise(v: Vector3<f32>, min: Vector3<f32>, max: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(
        v.x.max(min.x).min(max.x),
        v.y.max(min.y).min(max.y),
        v.z.max(min.z).min(max.z),
    )
}
