use bytemuck::Contiguous;
use cgmath::{Quaternion, Vector3};
use fastrand;
use super::BindGroups;
use super::wgpu::{self, Device, PipelineLayout, ShaderModule};

use super::{
    camera,
    primitives::{self, SDFPrimitive, Typus},
    // Input,
};

use crate::{x4,x3};

use self::distance::get_min_dst_to_primitives;

mod distance;

pub const VIEW_DST: f32 = 1000.0;

const PRIMITIVE_COUNT: u8 = 10;

struct SpawnData {
    last_spawn: std::time::Duration,
    min_spawn_time_s: f32,
    max_spawn_time_s: f32,
    next_random: f32,
    rng: fastrand::Rng,
}

impl SpawnData {
    fn new(rng: fastrand::Rng) -> Self {
        Self {
            last_spawn: std::time::Duration::from_secs(0),
            min_spawn_time_s: 0.5,
            max_spawn_time_s: 5.0,
            next_random: rng.f32(),
            rng,
        }
    }
    fn should_spawn(&mut self, dt: std::time::Duration, hardness: f32) -> bool {
        self.last_spawn += dt;
        let spawn_time = self.max_spawn_time_s
            - (self.max_spawn_time_s - self.min_spawn_time_s) * hardness * self.next_random;
        self.last_spawn.as_secs_f32() > spawn_time
    }
    fn did_spawn(&mut self) {
        self.last_spawn = std::time::Duration::from_secs(0);
        self.next_random = self.rng.f32();
    }
}

// struct Range {
//     min: f32,
//     len: f32,
// }
// const PRIMITIVE_SCALE_RANGE: Range = Range {
//     min: 1.0,
//     len: 20.0,
// };

pub(crate) struct SingleLevelManager {
    /// 0.0 - 1.0
    hardness: f32,
    rng: fastrand::Rng,
    pub primitive_manager: primitives::PrimitiveManager,
    pub camera: camera::RenderCamera,
    mouse_pressed: bool,
    total_time: std::time::Duration,
    spawn_data: SpawnData,
    pub game_over: bool,
}

impl SingleLevelManager {
    pub fn new(
        hardness: f32,
        seed: u64,
        device: &Device,
        size: (u32, u32),
    ) -> (Self, ShaderModule, PipelineLayout, BindGroups) {
        assert!(hardness >= 0.0 && hardness <= 1.0);
        let rng = fastrand::Rng::with_seed(seed);
        let (primitive_manager, primitives_bind_group) = primitives::PrimitiveManager::new(&device, PRIMITIVE_COUNT);
        let (camera, camera_bind_group) = camera::RenderCamera::new(device, size, VIEW_DST);
        let shader = device.create_shader_module(wgpu::include_wgsl!("level/main_shader.wgsl"));
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
                primitive_manager,
                camera,
                mouse_pressed: false,
                total_time: std::time::Duration::from_secs(0),
                spawn_data: SpawnData::new(rng.clone()),
                rng,
                game_over: false,
            },
            shader,
            render_pipeline_layout,
            BindGroups {
                primitives_bind_group,
                camera_bind_group,
            },
        )
    }

    pub fn start(&mut self, queue: &wgpu::Queue) {
        self.total_time = std::time::Duration::from_secs(0);
        let params = &RespawnParams {
            hardness: &self.hardness,
            rng: &self.rng,
        };
        self.primitive_manager.update_primitives(
            |primitives| {
                for primitive in primitives.iter_mut() {
                    if self.rng.f32() < self.hardness/0.6 {
                        respawn_primitive(params, primitive);
                    }
                }
            },
            queue,
        );
    }

    pub fn move_by(&mut self, delta: Vector3<f32>) {
        self.camera.controller.move_by(delta, &mut self.camera.camera); //weird controller but heyy
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        self.camera.resize(size.0, size.1);
    }
    pub fn update(&mut self, dt: std::time::Duration, queue: &wgpu::Queue) {
        self.total_time += dt;
        self.primitive_manager.update(dt, queue);
        self.camera.update(dt, queue);
        if self.spawn_data.should_spawn(dt, self.hardness) {
            if let Some(ref mut primitive) = self.primitive_manager.get_spawnable_primitive() {
                let params = &RespawnParams {
                    hardness: &self.hardness,
                    rng: &self.rng,
                };
                respawn_primitive(params, primitive);
                self.spawn_data.did_spawn();
            }
        };
        let dst = get_min_dst_to_primitives(self.camera.uniform.view_position, &mut self.primitive_manager.primitives);
        // println!("dst: {}", dst);
        if dst < 1.0 {
            self.game_over = true;
            println!("Game Over {}", dst);
        }
    }

    // pub(crate) fn input(&mut self, input: &Input) -> bool {
    //     match input {
    //         Input::Window(event) => match event {
    //             WindowEvent::KeyboardInput {
    //                 input:
    //                     KeyboardInput {
    //                         virtual_keycode: Some(key),
    //                         state,
    //                         ..
    //                     },
    //                 ..
    //             } => self.camera.controller.process_keyboard(*key, *state),
    //             WindowEvent::MouseWheel { delta, .. } => {
    //                 self.camera.projection.fovy = cgmath::num_traits::clamp(
    //                     self.camera.projection.fovy
    //                         + cgmath::Rad(
    //                             match delta {
    //                                 winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
    //                                 winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    //                             } / 5000.0,
    //                         ),
    //                     cgmath::Rad(0.3),
    //                     cgmath::Rad(3.0),
    //                 );
    //                 true
    //             }
    //             WindowEvent::MouseInput {
    //                 button: MouseButton::Left,
    //                 state,
    //                 ..
    //             } => {
    //                 self.mouse_pressed = *state == ElementState::Pressed;
    //                 true
    //             }
    //             _ => false,
    //         },
    //         Input::Device(event) => match event {
    //             DeviceEvent::MouseMotion { delta } => {
    //                 if self.mouse_pressed {
    //                     self.camera.controller.process_mouse(delta.0, delta.1);
    //                 }
    //                 true
    //             }
    //             _ => false,
    //         },
    //     }
    // }
}

struct RespawnParams<'a> {
    rng: &'a fastrand::Rng,
    hardness: &'a f32,
}

const DEBUG_PRIMITIVES : bool = false;
fn respawn_primitive(params: &RespawnParams, primitive: &mut SDFPrimitive) {
    let rng = params.rng;
    let hardness = *params.hardness;
    if DEBUG_PRIMITIVES {
        primitive.data = x4!(10.0);
        primitive.speed = 0.0;
        primitive.typus = Typus::Ellipsoid;
        primitive.place_in_spawn_area(rng);
        primitive.position[2] -= 1000.0;
    } else {

        primitive.data = x4!(hardness_to_scale(hardness, rng.f32()));
        primitive.speed = hardness_to_speed(hardness, rng.f32());
        let (u, v, w) = (rng.f32(), rng.f32(), rng.f32());
        let pi = std::f32::consts::PI;//∏
        primitive.rotation = [
            (1.0 - u).sqrt() * (2.0 * pi * v).sin(),
            (1.0 - u).sqrt() * (2.0 * pi * v).cos(),
            (u).sqrt() * (2.0 * pi * w).sin(),
            (u).sqrt() * (2.0 * pi * w).cos(),
            ];
            let rand_rot = || 0.1 * hardness * rng.f32();
            primitive.rotation_delta = Quaternion::from_arc(
                Vector3::unit_z(),
                Vector3::new(0.1 * rand_rot(), 0.1 * rand_rot(), 1.0 - 0.1 * rand_rot()),
                None,
            )
            .into();
        primitive.place_in_spawn_area(rng);
        primitive.rgba = x4!(rng.f32());
        let max_len = primitive.data.iter().fold(f32::MIN, |a, &b| a.max(b));
        const DISTANCE_FACTOR: f32 = 3.5;
        if max_len*3.0+DISTANCE_FACTOR*2.0 < VIEW_DST / 3.0 { //safety distance to prevent artifacts
            let triple_this_axis = || {
                (hardness > rng.f32()) as u32
            };
            primitive.instances = x3!(triple_this_axis());
            primitive.instances_distance = max_len*DISTANCE_FACTOR;
        } else {
            primitive.instances = x3!(0);
        }
        // primitive.twist = rng.f32()*5.0; //FIXME: doesnt work yet, disabled in shader rn
        
        //these integers are not in line with the ones used for enum representation, but that doesn't matter here
        match rng.u32(..=Typus::MAX_VALUE) {
            0 => {
                primitive.typus = Typus::Ellipsoid;
            }
            1 => {
                primitive.typus = Typus::BoxFrame;
                primitive.data[3] /= 15.0; //frame thickness
            }
            2 => {
                primitive.typus = Typus::Octahedron;
            }
            3 => {
                primitive.typus = Typus::ChainLink;
                primitive.data[2] /= 10.0; //link girth
                primitive.data[0] -= MIN_SCALE; //no length is fine as well thats a donut then
            }
            _ => {},//unreachable!(),
        }
    };
}

const MIN_SPEED: f32 = VIEW_DST / 20.0;
const MAX_SPEED: f32 = VIEW_DST / 2.0;

const MIN_SCALE: f32 = VIEW_DST / 100.0;
const MAX_SCALE: f32 = VIEW_DST / 4.0;

const MIN_X: f32 = -VIEW_DST;
const MAX_X: f32 = VIEW_DST;
const MIN_Y: f32 = -VIEW_DST;
const MAX_Y: f32 = VIEW_DST;
const MIN_Z: f32 = VIEW_DST + MAX_SCALE;
const MAX_Z: f32 = VIEW_DST + MAX_SCALE + 100.0;

fn hardness_to_speed(hardness: f32, random: f32) -> f32 {
    MIN_SPEED + hardness * random * (MAX_SPEED - MIN_SPEED)
}

fn hardness_to_scale(hardness: f32, random: f32) -> f32 {
    MIN_SCALE + hardness * random * (MAX_SCALE - MIN_SCALE)
}

impl SDFPrimitive {
    fn place_in_spawn_area(&mut self, rng: &fastrand::Rng) {
        self.position = [
            rng.f32() * (MAX_X - MIN_X) + MIN_X,
            rng.f32() * (MAX_Y - MIN_Y) + MIN_Y,
            rng.f32() * (MAX_Z - MIN_Z) + MIN_Z,
        ];
    }
}
