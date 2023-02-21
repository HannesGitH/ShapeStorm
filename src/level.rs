use bytemuck::Contiguous;
use fastrand;
use wgpu::{Device, PipelineLayout, ShaderModule};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, KeyboardInput, MouseButton, WindowEvent},
};

use crate::{
    camera,
    primitives::{self, SDFPrimitive, Typus},
    Input, x4,
};

const PRIMITIVE_COUNT: u8 = 10;

// struct Range {
//     min: f32,
//     len: f32,
// }
// const PRIMITIVE_SCALE_RANGE: Range = Range {
//     min: 1.0,
//     len: 20.0,
// };

pub(crate) struct LevelManager {
    hardness: f32,
    rng: fastrand::Rng,
    pub primitive_manager: primitives::PrimitiveManager,
    pub camera: camera::RenderCamera,
    mouse_pressed: bool,
}

impl LevelManager {
    pub fn new(
        hardness: f32,
        seed: u64,
        device: &Device,
        size: PhysicalSize<u32>,
    ) -> (Self, ShaderModule, PipelineLayout) {
        let rng = fastrand::Rng::with_seed(seed);
        let primitive_manager = primitives::PrimitiveManager::new(&device, PRIMITIVE_COUNT);
        let camera = camera::RenderCamera::new(&device, size);
        let shader = device.create_shader_module(wgpu::include_wgsl!("level.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &primitive_manager.bind_group_layout,
                    // &texture_bind_group_layout,
                    &camera.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        (
            Self {
                hardness,
                rng,
                primitive_manager,
                camera,
                mouse_pressed: false,
            },
            shader,
            render_pipeline_layout,
        )
    }

    pub fn start(&mut self, queue: &wgpu::Queue) {
        let params = &RespawnParams {
            hardness: &self.hardness,
            rng: &self.rng,
        };
        self.primitive_manager.update_primitives(
            |primitives| {
                for primitive in primitives.iter_mut() {
                    respawn_primitive(params, primitive);
                }
            },
            queue,
        );
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.camera.resize(size.width, size.height);
    }
    pub fn update(&mut self, dt: std::time::Duration, queue: &wgpu::Queue) {
        self.primitive_manager.update(dt, queue);
        self.camera.update(dt, queue);
    }

    pub(crate) fn input(&mut self, input: &Input) -> bool {
        match input {
            Input::Window(event) => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => self.camera.controller.process_keyboard(*key, *state),
                WindowEvent::MouseWheel { delta, .. } => {
                    self.camera.projection.fovy = cgmath::num_traits::clamp(
                        self.camera.projection.fovy
                            + cgmath::Rad(
                                match delta {
                                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                                } / 5000.0,
                            ),
                        cgmath::Rad(0.3),
                        cgmath::Rad(3.0),
                    );
                    true
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,
                    ..
                } => {
                    self.mouse_pressed = *state == ElementState::Pressed;
                    true
                }
                _ => false,
            },
            Input::Device(event) => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if self.mouse_pressed {
                        self.camera.controller.process_mouse(delta.0, delta.1);
                    }
                    true
                }
                _ => false,
            },
        }
    }
}

struct RespawnParams<'a> {
    rng: &'a fastrand::Rng,
    hardness: &'a f32,
}
fn respawn_primitive(params: &RespawnParams, primitive: &mut SDFPrimitive) {
    let rng = params.rng;
    let hardness = *params.hardness;
    primitive.data = x4!(hardness_to_scale(hardness,rng.f32()));
    primitive.speed = hardness_to_speed(hardness,rng.f32());
    primitive.place_in_spawn_area(rng);

    //these integers are not in line with the ones used for enum representation, but that doenst matter here
    match rng.u32(..Typus::MAX_VALUE) {
        0 => {
            primitive.typus = Typus::Ellipsoid;
        }
        1 => {
            primitive.typus = Typus::BoxFrame;
        }
        _ => {}
    };
}

fn hardness_to_speed(hardness: f32, random: f32) -> f32 {
    const MIN_SPEED: f32 = 1.0;
    const MAX_SPEED: f32 = 20.0;

    MIN_SPEED + hardness * random * (MAX_SPEED - MIN_SPEED)
}

fn hardness_to_scale(hardness: f32, random: f32) -> f32 {
    const MIN_SCALE: f32 = 1.0;
    const MAX_SCALE: f32 = 20.0;

    MIN_SCALE + hardness * random * (MAX_SCALE - MIN_SCALE)
}

impl SDFPrimitive {
    fn place_in_spawn_area(&mut self, rng: &fastrand::Rng) {
        const MIN_X : f32 = -1000.0;
        const MAX_X : f32 = 1000.0;
        const MIN_Y : f32 = -1000.0;
        const MAX_Y : f32 = 1000.0;
        const MIN_Z : f32 = 1000.0;
        const MAX_Z : f32 = 1100.0;

        self.position = [
            rng.f32() * (MAX_X - MIN_X) + MIN_X,
            rng.f32() * (MAX_Y - MIN_Y) + MIN_Y,
            rng.f32() * (MAX_Z - MIN_Z) + MIN_Z,
        ];
    }
}