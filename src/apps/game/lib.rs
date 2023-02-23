mod camera;
mod level;
mod primitives;
// mod texture;
mod macros;

use std::time::Duration;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;



struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer, //weg
    index_buffer: wgpu::Buffer,  //weg
    mouse_pressed: bool,
    scene: CurrentScene,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> State {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: surface.get_capabilities(&adapter).formats[..0].to_vec(),
        };
        surface.configure(&device, &config);

        let random_seed = fastrand::u64(..); //XXX: set according to level (from level-system)
        let (mut single_level_manager, shader, render_pipeline_layout) =
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
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

        //weg
        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        //weg
        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            &device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            clear_color: wgpu::Color::BLUE,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            // num_indices,
            // diffuse_bind_group,
            // diffuse_texture,
            mouse_pressed: false,
            scene,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            match &mut self.scene {
                CurrentScene::Level(ref mut single_level_manager) => {
                    single_level_manager.resize(new_size);
                }
                _ => {}
            }
        }
    }

    fn input(&mut self, input: Input) -> bool {
        if match &mut self.scene {
            CurrentScene::Level(single_level_manager) => single_level_manager.input(&input),
            CurrentScene::GameOver => false,//TODO: handleinputs in game over screen
        } {
            return true;
        }
        match input {
            Input::Window(event) => match event {
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
            Input::Device(_) => false,
        }
    }

    fn update(&mut self, dt: Duration) {
        match &mut self.scene {
            CurrentScene::Level(single_level_manager) => {
                single_level_manager.update(dt, &self.queue);
                if single_level_manager.game_over {
                    self.scene = CurrentScene::GameOver;
                }
            }
            CurrentScene::GameOver => {},
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            match &mut self.scene {
                CurrentScene::Level(single_level_manager) => {
                    render_pass.set_bind_group(1, &single_level_manager.camera.bind_group, &[]);
                    render_pass.set_bind_group(0, &single_level_manager.primitive_manager.bind_group, &[]);
                }
                CurrentScene::GameOver => {
                    //TODO: render game over screen
                },
            }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);

            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.draw(0..5, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

enum Input<'a> {
    Device(&'a DeviceEvent),
    Window(&'a WindowEvent<'a>),
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;
    let mut last_render_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            let now = std::time::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;
            state.update(dt);
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        Event::DeviceEvent { event, .. } => {
            if !state.input(Input::Device(&event)) {
                match event {
                    _ => {}
                }
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(Input::Window(event)) {
                match event {
                    #[cfg(not(target_arch = "wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}

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
