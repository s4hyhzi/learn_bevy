use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalSize;
use winit::keyboard::{Key, NamedKey};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use crate::gui_tools::GuiRenderer;

pub trait Application {
    async fn start(&self) {
        let event_loop = EventLoop::new().unwrap();

        let (window, size, surface, _adapter, device, queue) = {
            let builder = WindowBuilder::new();
            let window = Arc::new(builder
                .with_title("Hello Wgpu!")
                .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
                .with_resizable(false)
                .with_maximized(false)
                .build(&event_loop).unwrap());
            let size = window.inner_size();

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

            let mut features = wgpu::Features::empty();
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


            (window, size, surface, adapter, device, queue)
        };
        let (mut config, mut gui_renderer) = create_egui(&window, &size, &surface, &_adapter, &device);

        let mut close_requested = false;
        let mut modifiers = ModifiersState::default();

        let mut scale_factor = 1.0;
        let mut view_update = false;

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => {
                    gui_renderer.handle_input(&window, &event);

                    match event {
                        WindowEvent::CloseRequested => {
                            close_requested = true;
                        }
                        WindowEvent::ModifiersChanged(new) => {
                            modifiers = new.state();
                        }
                        WindowEvent::KeyboardInput {
                            event: kb_event, ..
                        } => {
                            if kb_event.logical_key == Key::Named(NamedKey::Escape) {
                                close_requested = true;
                                return;
                            }
                        }
                        WindowEvent::Resized(new_size) => {
                            // Resize surface:
                            if new_size.width > 0 && new_size.height > 0 {
                                config.width = new_size.width;
                                config.height = new_size.height;
                                surface.configure(&device, &config);
                                view_update = true;
                            } else {
                                view_update = false;
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            if view_update {
                                let surface_texture = surface
                                    .get_current_texture()
                                    .expect("Failed to acquire next swap chain texture");

                                let surface_view = surface_texture
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());

                                let mut encoder =
                                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                        label: None,
                                    });

                                let screen_descriptor = ScreenDescriptor {
                                    size_in_pixels: [config.width, config.height],
                                    pixels_per_point: window.scale_factor() as f32 * scale_factor,
                                };

                                gui_renderer.draw(
                                    &device,
                                    &queue,
                                    &mut encoder,
                                    &window,
                                    &surface_view,
                                    screen_descriptor,
                                    |ctx| {
                                        egui::Window::new("hello world!")
                                            .resizable(true)
                                            .vscroll(true)
                                            .default_open(false)
                                            .show(&ctx, |mut ui| {
                                                ui.label("Label!");

                                                if ui.button("Button!").clicked() {
                                                    println!("boom!")
                                                }

                                                ui.separator();
                                                ui.horizontal(|ui| {
                                                    ui.label(format!(
                                                        "Pixels per point: {}",
                                                        ctx.pixels_per_point()
                                                    ));
                                                    if ui.button("-").clicked() {
                                                        scale_factor = (scale_factor - 0.1).max(0.3);
                                                    }
                                                    if ui.button("+").clicked() {
                                                        scale_factor = (scale_factor + 0.1).min(3.0);
                                                    }
                                                });
                                            });
                                    },
                                );

                                queue.submit(Some(encoder.finish()));
                                surface_texture.present();
                            }

                            window.request_redraw();
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    if close_requested {
                        elwt.exit()
                    }
                }
                _ => {}
            }
        }).expect("TODO: panic message");
    }

    fn update(&self);
}

fn create_egui(window: &Window, size: &PhysicalSize<u32>, surface: &Surface, adapter: &Adapter, device: &Device) -> (SurfaceConfiguration, GuiRenderer) {
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let selected_format = TextureFormat::Bgra8UnormSrgb;
    let swapchain_format = swapchain_capabilities
        .formats
        .iter()
        .find(|d| **d == selected_format)
        .expect("failed to select proper surface texture format!");
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *swapchain_format,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 0,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
        width: size.width as u32,
        height: size.height as u32,
    };

    surface.configure(&device, &config);
    let mut gui_renderer = GuiRenderer::new(&device, config.format, None, 1, &window);
    (config, gui_renderer)
}