use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};

use crate::data_stuct::{Pass, State};
use crate::utils::glsl_to_wgsl;

#[allow(dead_code)]
#[derive(Debug)]
struct WindowState {
    close_requested: bool,
    view_updated: bool,
    factor: f64,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Application {
    window: Arc<Window>,
    window_state: WindowState,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    last_frame_time: Instant,
    states: Option<State>,
}


impl Application {
    pub async fn new() {
        let event_loop = EventLoop::new().unwrap();
        println!("creating");
        let mut app = Self::create(&event_loop).await;
        app.init_render_passes();
        println!("created");
        event_loop.run(move |event, elwt| {
            app.event_handler(event, elwt);
        }).expect("Failed to run event loop");
    }

    async fn create(event_loop: &EventLoop<()>) -> Self {
        println!("Creating Application");
        let mut window_state = WindowState {
            close_requested: false,
            view_updated: false,
            factor: 1.0,
        };

        let builder = WindowBuilder::new();
        let window = Arc::new(builder
            .with_title("Hello Wgpu!")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .with_min_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .build(&event_loop).unwrap());
        let size = window.inner_size();
        window_state.factor = window.scale_factor();

        // instance 变量是 GPU 实例
        // Backends::all 对应 Vulkan、Metal、DX12、WebGL 等所有后端图形驱动
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
            width: size.width,
            height: size.height,
        };

        surface.configure(&device, &config);
        surface.get_current_texture().unwrap();
        let app = Self {
            window,
            window_state,
            surface,
            device,
            queue,
            config,
            size,
            last_frame_time: Instant::now(),
            states: None,
        };
        app
    }

    pub fn event_handler(&mut self, event: winit::event::Event<()>, elwt: &EventLoopWindowTarget<()>) {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.window_state.close_requested = true;
                    }
                    winit::event::WindowEvent::Resized(new_size) => {
                        if new_size.width > 0 && new_size.height > 0 {
                            self.size = new_size;
                            self.resize();
                            self.window_state.view_updated = true;
                        } else {
                            self.window_state.view_updated = false;
                        }
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.window_state.factor = scale_factor;
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        if self.window_state.view_updated {
                            self.redraw();
                        }
                    }
                    _ => {}
                }
            }
            winit::event::Event::AboutToWait { .. } => {
                if self.window_state.close_requested {
                    elwt.exit();
                }
            }
            _ => {}
        }
    }

    pub fn init_render_passes(&mut self) {
        println!("Initializing");
        let vs_code = glsl_to_wgsl(include_str!("../assets/shader.vert"), naga::ShaderStage::Vertex);
        let vs_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(vs_code.into()),
        });
        let fs_code = glsl_to_wgsl(include_str!("../assets/shader.frag"), naga::ShaderStage::Fragment);
        let fs_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(fs_code.into()),
        });

        let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                compilation_options: Default::default(),
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let pass = Pass {
            pipeline: render_pipeline,
        };

        self.states = Some(State {
            forward_pass: pass,
        });
    }

    pub fn redraw(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_frame_time) < Duration::from_millis(16) {
            return;
        }
        println!("Redrawing");

        self.last_frame_time = now;

        if !self.window.is_visible().unwrap_or(false) {
            return;
        }
        let frame = self.surface.get_current_texture().unwrap();
        let surface_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // 这就是片元着色器中 @location(0) 标记指向的颜色附件
                    Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: wgpu::StoreOp::Store
                        }
                    })
                ],
                ..Default::default()
            });

            // 新添加!
            let Some(passes) = &self.states else { return };
            render_pass.set_pipeline(&passes.forward_pass.pipeline); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.window.request_redraw();
    }

    pub fn resize(&mut self) {
        println!("Resizing");
        self.size = self.window.inner_size();
        if self.size.width > 0 && self.size.height > 0 {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.reconfigure_surface();
    }

    fn reconfigure_surface(&mut self) {
        self.size = self.window.inner_size();
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        self.surface.configure(&self.device, &self.config);
    }
}