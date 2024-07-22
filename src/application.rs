use std::sync::Arc;
use futures::executor::block_on;
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::Window;

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
}

impl Application {
    pub fn new() {
        let event_loop = EventLoop::new().unwrap();
        let mut app = Self::create(&event_loop);
        event_loop.run(move |event, elwt| {
            app.event_handler(event, elwt);
        }).expect("Failed to run event loop");
    }

    fn create(event_loop:&EventLoop<()>) -> Self {
        println!("Creating Application");
        let mut window_state = WindowState {
            close_requested: false,
            view_updated: false,
            factor: 1.0,
        };

        let builder = winit::window::WindowBuilder::new()
            .with_title("webgpu-rs")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080));

        let window = Arc::new(builder.build(&event_loop).unwrap());
        let size = window.inner_size();
        window_state.factor = window.scale_factor();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
        ).expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM | wgpu::Features::SPIRV_SHADER_PASSTHROUGH;

        let (device, queue) = block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                }, None)
        ).expect("Failed to create device");

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
        let app = Self {
            window,
            window_state,
            surface,
            device,
            queue,
            config,
            size,
        };
        app
    }

    pub fn event_handler(&mut self, event: winit::event::Event<()>, elwt: &EventLoopWindowTarget<()>) {
        // let Application { window_state, window, .. } = self;
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.window_state.close_requested = true;
                    }
                    winit::event::WindowEvent::Resized(new_size) => {
                        if new_size.width > 0 && new_size.height > 0 {
                            self.window_state.view_updated = true;
                            self.resize();
                        }else {
                            self.window_state.view_updated = false;
                        }
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.window_state.factor = scale_factor;
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        if self.window_state.view_updated {
                            self.redraw();
                            self.window.request_redraw();
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

    pub fn redraw(&self){
        println!("Redrawing");
    }

    pub fn resize(&self){
        println!("Resizing");
    }
}