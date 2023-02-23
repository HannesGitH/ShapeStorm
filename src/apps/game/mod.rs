use std::{num::NonZeroU64, sync::Arc, time::{Duration, Instant}};

use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};
use egui::Window;

mod camera;
mod level;
mod macros;
mod primitives;

enum CurrentScene {
    Level(level::SingleLevelManager),
    GameOver,
    // Menu,
}

pub(crate) struct State {
    // surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    // config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    clear_color: wgpu::Color,
    // render_pipeline: wgpu::RenderPipeline,
    mouse_pressed: bool,
    scene: CurrentScene,
    last_time: Option<Instant>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let (device, queue) = (
            wgpu_render_state.device.clone(),
            wgpu_render_state.queue.clone(),
        );

        let size_vec2 = // cc.integration_info.window_info.size; //thats the full window size..
        // cc.egui_ctx.used_size();
        cc.egui_ctx.screen_rect().size();
        let size: (u32, u32) = (size_vec2.x as u32, size_vec2.y as u32);

        let random_seed = fastrand::u64(..); //XXX: set according to level (from level-system)
        let (mut single_level_manager, shader, render_pipeline_layout, bind_groups) =
            level::SingleLevelManager::new(0.7, random_seed, &device, size);
        single_level_manager.start(&queue);

        let scene = CurrentScene::Level(single_level_manager);

        //XXX: put that in the level man or a state match block?
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())], //ah-ha
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(GameRendering {
                render_pipeline,
                bind_groups,
                // diffuse_bind_group,
                // diffuse_texture,
            });

        Some(Self {
            // surface,
            device,
            queue,
            size,
            clear_color: wgpu::Color::BLUE,
            // render_pipeline,
            // num_indices,
            // diffuse_bind_group,
            // diffuse_texture,
            mouse_pressed: false,
            scene,
            last_time: None,
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            // self.config.width = new_size.0;
            // self.config.height = new_size.1;
            // self.surface.configure(&self.device, &self.config);
            match &mut self.scene {
                CurrentScene::Level(ref mut single_level_manager) => {
                    single_level_manager.resize(new_size);
                }
                _ => {}
            }
        }
    }

    // fn input(&mut self, input: Input) -> bool {
    //     if match &mut self.scene {
    //         CurrentScene::Level(single_level_manager) => single_level_manager.input(&input),
    //         CurrentScene::GameOver => false,//TODO: handleinputs in game over screen
    //     } {
    //         return true;
    //     }
    //     match input {
    //         Input::Window(event) => match event {
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
    //         Input::Device(_) => false,
    //     }
    // }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = if let Some(lt)=self.last_time{
            now.duration_since(lt)}else{Duration::from_secs(0)};
        self.last_time = Some(now);
        match &mut self.scene {
            CurrentScene::Level(single_level_manager) => {
                single_level_manager.update(dt, &self.queue);
                if single_level_manager.game_over {
                    self.scene = CurrentScene::GameOver;
                }
            }
            CurrentScene::GameOver => {}
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update();
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }
}

impl State {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        // let angle += response.drag_delta().x * 0.01;

        self.resize((rect.width() as u32, rect.height() as u32));
        // let time_delta = response.ctx

        match &mut self.scene {
            CurrentScene::Level(single_level_manager) => {}
            CurrentScene::GameOver => {
                //TODO: render game over screen
            }
        }

        // The callback function for WGPU is in two stages: prepare, and paint.
        //
        // The prepare callback is called every frame before paint and is given access to the wgpu
        // Device and Queue, which can be used, for instance, to update buffers and uniforms before
        // rendering.
        //
        // You can use the main `CommandEncoder` that is passed-in, return an arbitrary number
        // of user-defined `CommandBuffer`s, or both.
        // The main command buffer, as well as all user-defined ones, will be submitted together
        // to the GPU in a single call.
        //
        // The paint callback is called after prepare and is given access to the render pass, which
        // can be used to issue draw commands.
        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                // let resources = paint_callback_resources.get().unwrap();
                // resources.prepare(device, queue, angle);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &GameRendering = paint_callback_resources.get().unwrap();
                resources.render(render_pass);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

pub(crate) struct BindGroups {
    camera_bind_group: wgpu::BindGroup,
    primitives_bind_group: wgpu::BindGroup,
}
struct GameRendering {
    render_pipeline: wgpu::RenderPipeline,
    bind_groups: BindGroups,
}

impl GameRendering {
    fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.render_pipeline);
        // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

        render_pass.set_bind_group(1, &self.bind_groups.camera_bind_group, &[]);
        render_pass.set_bind_group(0, &self.bind_groups.primitives_bind_group, &[]);

        // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        render_pass.draw(0..6, 0..1);
    }
}

// enum Input<'a> {
//     Device(&'a DeviceEvent),
//     Window(&'a WindowEvent<'a>),
// }

// pub async fn run() {
//     env_logger::init();
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();

//     let mut state = State::new(window).await;
//     let mut last_render_time = std::time::Instant::now();

//     event_loop.run(move |event, _, control_flow| match event {
//         Event::RedrawRequested(window_id) if window_id == state.window().id() => {
//             let now = std::time::Instant::now();
//             let dt = now - last_render_time;
//             last_render_time = now;
//             state.update(dt);
//             match state.render() {
//                 Ok(_) => {}
//                 // Reconfigure the surface if lost
//                 Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
//                 // The system is out of memory, we should probably quit
//                 Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                 // All other errors (Outdated, Timeout) should be resolved by the next frame
//                 Err(e) => eprintln!("{:?}", e),
//             }
//         }
//         Event::MainEventsCleared => {
//             // RedrawRequested will only trigger once, unless we manually
//             // request it.
//             state.window().request_redraw();
//         }
//         Event::DeviceEvent { event, .. } => {
//             if !state.input(Input::Device(&event)) {
//                 match event {
//                     _ => {}
//                 }
//             }
//         }
//         Event::WindowEvent {
//             ref event,
//             window_id,
//         } if window_id == state.window().id() => {
//             if !state.input(Input::Window(event)) {
//                 match event {
//                     #[cfg(not(target_arch = "wasm32"))]
//                     WindowEvent::CloseRequested
//                     | WindowEvent::KeyboardInput {
//                         input:
//                             KeyboardInput {
//                                 state: ElementState::Pressed,
//                                 virtual_keycode: Some(VirtualKeyCode::Escape),
//                                 ..
//                             },
//                         ..
//                     } => *control_flow = ControlFlow::Exit,
//                     WindowEvent::Resized(physical_size) => {
//                         state.resize(*physical_size);
//                     }
//                     WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                         state.resize(**new_inner_size);
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         _ => {}
//     });
// }

// MARK: useless stuff

// vertices

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    // tex_coords: [f32; 2], // NEW!
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // wgpu::VertexAttribute {
                //     offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                //     shader_location: 1,
                //     format: wgpu::VertexFormat::Float32x2,
                // },
            ],
        }
    }
}

const VERTICES: &[Vertex; 4] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
];

const INDICES: &[u16] = &[2, 1, 0, 3, 2, 0];
