use cgmath::*;
use super::wgpu::{self, Device, Queue};
use super::wgpu::util::DeviceExt;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
// use winit::dpi::{PhysicalSize};
// use winit::event::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


#[repr(u32)]
#[derive(Debug, Copy, Clone)] //, bytemuck::Pod, bytemuck::Zeroable)]
pub enum Effect {
    None,
    GlassyOnion,
    GlowOff,
    CleanFromWater,
    BlackBody,
    WhiteBody,
    // HalfStep,
}
// unsafe impl bytemuck::Contiguous for Effect {
//     type Int = u32;
//     const MIN_VALUE: u32 = Effect::None as u32;
//     const MAX_VALUE: u32 = Effect::HalfStep as u32;
// }
unsafe impl bytemuck::Zeroable for Effect {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for Effect {}

impl Default for Effect {
    fn default() -> Self {
        Effect::None
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    pixels : (u32, u32),
    pub fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            pixels: (width, height),
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels = (width, height);
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let aspect = self.pixels.0 as f32 / self.pixels.1 as f32;
        OPENGL_TO_WGPU_MATRIX * 
        perspective(self.fovy, aspect, self.znear, self.zfar)
    }

    pub fn get_pixel_normalization_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(-0.5, 0.5, 0.0)) * Matrix4::from_nonuniform_scale(1.0/(self.pixels.0 as f32), -1.0/(self.pixels.1 as f32), 1.0) //* Matrix4::from_translation(Vector3::new(-0.5*(self.pixels.0 as f32), 0.5, 0.0))
    }

}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    // scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            // scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    // pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
    //     let amount = if state == ElementState::Pressed {
    //         1.0
    //     } else {
    //         0.0
    //     };
    //     match key {
    //         VirtualKeyCode::W | VirtualKeyCode::Up => {
    //             self.amount_up = amount;
    //             true
    //         }
    //         VirtualKeyCode::S | VirtualKeyCode::Down => {
    //             self.amount_down = amount;
    //             true
    //         }
    //         VirtualKeyCode::A | VirtualKeyCode::Left => {
    //             self.amount_left = amount;
    //             true
    //         }
    //         VirtualKeyCode::D | VirtualKeyCode::Right => {
    //             self.amount_right = amount;
    //             true
    //         }
    //         // VirtualKeyCode::Space => {
    //         //     self.amount_forward = amount;
    //         //     true
    //         // }
    //         // VirtualKeyCode::LShift => {
    //         //     self.amount_backward = amount;
    //         //     true
    //         // }
    //         _ => false,
    //     }
    // }

    pub fn process_mouse(&mut self, _mouse_dx: f64,_mouse_dyy: f64) {
        // self.rotate_horizontal = mouse_dx as f32;
        // self.rotate_vertical = mouse_dy as f32;
    }

    // pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
    //     self.scroll = match delta {
    //         // I'm assuming a line is about 100 pixels
    //         MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
    //         MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
    //     };
    // }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position = (camera.position + forward * (self.amount_forward - self.amount_backward) * self.speed * dt)%super::level::VIEW_DST; //XXX: did it work?
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [1.0; 4],
            world_to_screen: cgmath::Matrix4::identity().into(),
            screen_to_world: cgmath::Matrix4::identity().into(),
            pixel_normalization_matrix: cgmath::Matrix4::identity().into(),
            effect : Effect::default(),
            _pad: [0.0; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub(crate) view_position: [f32; 4],
    world_to_screen: [[f32; 4]; 4],
    screen_to_world: [[f32; 4]; 4],
    pixel_normalization_matrix : [[f32; 4]; 4],
    pub effect : Effect,
    _pad: [f32; 3],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [1.0; 4],
            world_to_screen: cgmath::Matrix4::identity().into(),
            screen_to_world: cgmath::Matrix4::identity().into(),
            pixel_normalization_matrix: cgmath::Matrix4::identity().into(),
            // effect : Effect::CleanFromWater,
            // effect : Effect::GlowOff,
            // effect : Effect::WhiteBody,
            // effect : Effect::GlassyOnion,
            ..Default::default()
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        let proj = projection.calc_matrix();
        let world_to_cam = camera.calc_matrix();
        self.world_to_screen = (proj * world_to_cam).into();
        self.screen_to_world = //(camera.calc_inverse_matrix() * proj.invert().unwrap()).into();
            (
                // camera.calc_inverse_matrix() * 
                world_to_cam.invert().unwrap() 
                * 
                proj.invert().unwrap() 
            // * projection.get_uv_to_screen_matrix()
        ).into();
        self.pixel_normalization_matrix = projection.get_pixel_normalization_matrix().into();
    }
}

pub struct RenderCamera {
    pub camera: Camera,
    pub projection: Projection,
    pub controller: CameraController,
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl RenderCamera {
    pub fn new(device : &Device, size: (u32,u32), far:f32)->Self{

        let camera = Camera::new((0.0, 0.0, 0.0), cgmath::Deg(90.0), cgmath::Deg(0.0));
        let projection =
            Projection::new(size.0, size.1, cgmath::Deg(120.0), 1.0, far);
        let controller = CameraController::new(far, 0.5);

        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&camera, &projection);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        Self{
            camera,
            projection,
            controller,
            uniform,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
    fn update_controller(&mut self, dt: Duration) {
        self.controller.update_camera(&mut self.camera, dt);
    }
    fn update_uniform(&mut self, queue: &Queue) {
        self.uniform.update_view_proj(&self.camera, &self.projection);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
    pub fn update(&mut self, dt: Duration, queue: &Queue) {
        self.update_controller(dt);
        self.update_uniform(queue);
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
    }
}
