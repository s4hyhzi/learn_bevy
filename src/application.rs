use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use wgpu::{Adapter, Device, ShaderModuleDescriptorSpirV, Surface, SurfaceConfiguration, TextureFormat};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::dpi::PhysicalSize;
use winit::keyboard::{Key, NamedKey};
use winit::keyboard::ModifiersState;
use winit::window::Window;

use crate::{load_glsl, ShaderStage};
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

            let mut features = wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
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

        // let shader_vs = include_spirv_raw!("shader.vert.spv");
        // let vs_module = unsafe { device.create_shader_module_spirv(&shader_vs) };

        // let shader_fs = include_spirv_raw!("shader.frag.spv");
        // let fs_module = unsafe { device.create_shader_module_spirv(&shader_fs) };

        let shader_source = load_glsl(include_str!("shader.vert"), ShaderStage::Vertex);
        let shader_vs_raw = wgpu::util::make_spirv_raw(&shader_source);
        let shader_vs = ShaderModuleDescriptorSpirV {
            label: Some("shader.vert.spv"),
            source: shader_vs_raw,
        };
        let vs_module = unsafe { device.create_shader_module_spirv(&shader_vs) };

        let shader_source = load_glsl(include_str!("shader.frag"), ShaderStage::Fragment);
        let shader_fs_raw = wgpu::util::make_spirv_raw(&shader_source);
        let shader_fs = ShaderModuleDescriptorSpirV {
            label: Some("shader.frag.spv"),
            source: shader_fs_raw,
        };
        let fs_module = unsafe { device.create_shader_module_spirv(&shader_fs) };

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries: &[] });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],

            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });


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
                                        label: Some("Render Encoder"),
                                    });
                                {
                                    
                                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("Render Pass"),
                                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                            view: &(surface_view),
                                            resolve_target: None,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                                    r: 0.1,
                                                    g: 0.2,
                                                    b: 0.3,
                                                    a: 1.0,
                                                }),
                                                store: wgpu::StoreOp::Store,
                                            },
                                        })],
                                        ..Default::default()
                                    });
                                    
                                    render_pass.set_pipeline(&render_pipeline);
                                    render_pass.set_bind_group(0, &bind_group, &[]);
                                    render_pass.draw(0..3, 0..1);
                                }
                                
                                {
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
                                }
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
        width: size.width,
        height: size.height,
    };

    surface.configure(&device, &config);
    let mut gui_renderer = GuiRenderer::new(&device, config.format, None, 1, &window);
    (config, gui_renderer)
}