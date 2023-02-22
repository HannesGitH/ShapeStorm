use std::{time::Duration};

use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

#[repr(u32)]
#[derive(Debug, Copy, Clone)] //, bytemuck::Pod, bytemuck::Zeroable)]
pub enum Typus {
    BoxFrame,
    Ellipsoid,
    // Cube,
    // Torus,
    // Cylinder,
    // Cone,
    // Capsule,
    // Plane,
    // Triangle,
    // Pyramid,
    // Icosahedron,
    // Dodecahedron,
    // Octahedron,
    // Tetrahedron,
    // Custom,
}
unsafe impl bytemuck::Contiguous for Typus {
    type Int = u32;
    const MIN_VALUE: u32 = Typus::BoxFrame as u32;
    const MAX_VALUE: u32 = Typus::Ellipsoid as u32;
}
unsafe impl bytemuck::Zeroable for Typus {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for Typus {}

impl Default for Typus {
    fn default() -> Self {
        Typus::BoxFrame
    }
}

// #[repr(C , align(16))]
// the paddings allow aliognment of 16bytes for my actual variables
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SDFPrimitive {
    pub position: [f32; 3],
    pub speed: f32,
    pub rotation: [f32; 4],
    pub rotation_delta: [f32; 4],
    pub data: [f32; 4],
    pub instances: [u32; 3],
    pub instances_distance: f32,
    pub rgba: [f32; 4],
    pub typus: Typus,
    _pad4: [f32; 3],
    // operation: u32,
    // blend_strength: f32,
    // filler: [u32; 5], // 32 byte alignment
}

impl SDFPrimitive {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, -10000.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            rotation_delta: [0.0, 0.0, 0.0, 1.0],
            // typus: Typus::Sphere,
            ..Default::default()
        }
    }
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
// struct Primitives {
//     primitives: [SDFPrimitive; 10],
// }

pub struct PrimitiveManager {
    pub primitives: Vec<SDFPrimitive>,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    // total_time: Duration,
}

impl PrimitiveManager {
    pub fn new(device: &Device, primitive_count: u8) -> Self {
        let (bind_group, bind_group_layout, buffer) =
            mk_primitive_bind_group(device, primitive_count);

        let primitives = vec![SDFPrimitive::new(); primitive_count as usize];

        Self {
            primitives: primitives,
            buffer,
            bind_group,
            bind_group_layout,
            // total_time: Duration::from_secs(0),
        }
    }
    pub fn update_primitives<F>(&mut self, primitive_updater: F, queue: &wgpu::Queue)
    where
        F: Fn(&mut Vec<SDFPrimitive>),
    {
        primitive_updater(&mut self.primitives);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.primitives));
    }
    pub fn update(&mut self, dt: Duration, queue: &wgpu::Queue) {
        // self.total_time += dt;
        // let total_time = self.total_time;
        let updater = |primitives: &mut Vec<SDFPrimitive>| {
            for primitive in primitives.iter_mut() {
                primitive.position[2] -= primitive.speed * dt.as_secs_f32();
                // let (v0, v1) = (Simd::from(primitive.rotation), Simd::from(primitive.rotation_delta));
                // primitive.rotation = (v0 * v1).into();
                primitive.rotation = (cgmath::Quaternion::from(primitive.rotation)
                    * cgmath::Quaternion::from(primitive.rotation_delta)).into();
            }
        };
        self.update_primitives(updater, queue)
    }

    pub fn get_spawnable_primitive(&mut self) -> Option<&mut SDFPrimitive> {
        self.primitives.iter_mut().find(|primitive| {
            primitive.position[2] < -1000.0
        })
    }
}

fn mk_primitive_bind_group(
    device: &Device,
    primitive_count: u8,
) -> (BindGroup, BindGroupLayout, Buffer) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Primitives Buffer"),
        contents: bytemuck::cast_slice(&vec![SDFPrimitive::new(); primitive_count as usize]),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("primitives_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("primitives_bind_group"),
    });
    (bind_group, layout, buffer)
}
