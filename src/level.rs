use fastrand;
use wgpu::{Device, PipelineLayout, ShaderModule};
use winit::{dpi::PhysicalSize, event::{WindowEvent, ElementState, KeyboardInput, MouseButton, DeviceEvent}};

use crate::{camera, primitives::{self, Typus}, Input};

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
        let mut primitive_manager = primitives::PrimitiveManager::new(&device, 2);
        primitive_manager.primitives[0].typus = Typus::Sphere;
        primitive_manager.primitives[0].position = [0.0, 0.0, 800.0];
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
                    self.camera.projection.fovy = cgmath::num_traits::clamp(self.camera.projection.fovy + cgmath::Rad(match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    }/5000.0), cgmath::Rad(0.3), cgmath::Rad(3.0));
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
