use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use wgpu::{Device, StencilFaceState, StencilState, SurfaceConfiguration, TextureFormat};
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

use crate::gui_tools::GuiRenderer;
use crate::temp::*;
use crate::vertex::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[allow(dead_code)]
pub fn cast_slice<T>(data: &[T]) -> &[u8] {
    use std::mem::size_of;
    use std::slice::from_raw_parts;

    unsafe { from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
}

pub struct Application {
    entities: Vec<Entity>,
    lights: Vec<Light>,
    lights_are_dirty: bool,
    shadow_pass: Pass,
    forward_pass: Pass,
    forward_depth: wgpu::TextureView,
    light_uniform_buf: wgpu::Buffer,
}

impl Application {
    const MAX_LIGHTS: usize = 10;
    const SHADOW_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    const SHADOW_SIZE: wgpu::Extent3d = wgpu::Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 2,
    };
    const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn init(config: &SurfaceConfiguration, device: &Device) {
        let vertex_size = std::mem::size_of::<Vertex>();
        let (cube_vertex_data, cube_index_data) = create_cube();

        let cube_vertex_buf = Rc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: cast_slice(&cube_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        }));

        let cube_index_buf = Rc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: cast_slice(&cube_index_data),
            usage: wgpu::BufferUsages::INDEX,
        }));

        let (plane_vertex_data, plane_index_data) = create_plane();

        let plane_vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: cast_slice(&plane_vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let plane_index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: cast_slice(&plane_index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let entity_uniform_size = std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
        let plane_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Uniform Buffer"),
            size: entity_uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let local_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Local Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let mut enities = vec![{
            use cgmath::SquareMatrix;

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Entity Bind Group"),
                layout: &local_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &plane_uniform_buf,
                            offset: 0,
                            size: Some(entity_uniform_size.try_into().unwrap()),
                        }),
                    },
                ],
            });

            Entity {
                mx_world: cgmath::Matrix4::identity(),
                rotation_speed: 0.0,
                color: wgpu::Color::WHITE,
                vertex_buf: Rc::new(plane_vertex_buf),
                index_buf: Rc::new(plane_index_buf),
                index_count: 0,
                bind_group,
                uniform_buf: plane_uniform_buf,
            }
        }];

        struct CubeDesc {
            offset: cgmath::Vector3<f32>,
            angle: f32,
            scale: f32,
            rotation: f32,
        }
        let cube_descs = [
            CubeDesc {
                offset: cgmath::vec3(-2.0, -2.0, 2.0),
                angle: 10.0,
                scale: 0.7,
                rotation: 0.1,
            },
            CubeDesc {
                offset: cgmath::vec3(2.0, -2.0, 2.0),
                angle: 50.0,
                scale: 1.3,
                rotation: 0.2,
            },
            CubeDesc {
                offset: cgmath::vec3(-2.0, 2.0, 2.0),
                angle: 140.0,
                scale: 1.1,
                rotation: 0.3,
            },
            CubeDesc {
                offset: cgmath::vec3(2.0, 2.0, 2.0),
                angle: 210.0,
                scale: 0.9,
                rotation: 0.4,
            },
        ];

        for cube_desc in cube_descs {
            use cgmath::{Decomposed, Deg, InnerSpace, Quaternion, Rotation3};

            let transform = Decomposed {
                disp: cube_desc.offset.clone(),
                rot: Quaternion::from_axis_angle(cube_desc.offset.normalize(), Deg(cube_desc.angle)),
                scale: cube_desc.scale,
            };
            let uniform_size = std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
            let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Entity Uniform Buffer"),
                size: entity_uniform_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            enities.push(Entity {
                mx_world: transform.into(),
                rotation_speed: cube_desc.rotation,
                color: wgpu::Color::WHITE,
                vertex_buf: Rc::clone(&cube_vertex_buf),
                index_buf: Rc::clone(&cube_index_buf),
                index_count: cube_index_data.len(),
                bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Entity Bind Group"),
                    layout: &local_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &uniform_buf,
                                offset: 0,
                                size: Some(uniform_size.try_into().unwrap()),
                            }),
                        },
                    ],
                }),
                uniform_buf,
            });
        }

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Texture"),
            size: Self::SHADOW_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut shadow_target_views = (0..2).map(|i| {
            Some(shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Shadow Target View"),
                format: Some(Self::SHADOW_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: i,
                array_layer_count: Some(1),
            }))
        }).collect::<Vec<_>>();

        let lights = vec![
            Light {
                pos: cgmath::Point3::new(7.0, -5.0, 10.0),
                color: wgpu::Color {
                    r: 0.5,
                    g: 1.0,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 60.0,
                depth: 1.0..20.0,
                target_view: shadow_target_views[0].take().unwrap(),
            },
            Light {
                pos: cgmath::Point3::new(-5.0, 7.0, 10.0),
                color: wgpu::Color {
                    r: 1.0,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                fov: 45.0,
                depth: 1.0..20.0,
                target_view: shadow_target_views[1].take().unwrap(),
            },
        ];
        let light_uniform_size = std::mem::size_of::<LightRaw>() as wgpu::BufferAddress;
        let light_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: light_uniform_size * Self::MAX_LIGHTS as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vb_desc = wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 3 * 4,
                    shader_location: 1,
                },
            ],
        };

        let shadow_pass = {
            let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: Default::default(),
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout],
                push_constant_ranges: &[],
            });

            let uniform_size = std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
            let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Uniform Buffer"),
                size: uniform_size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shadow Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform_buf,
                            offset: 0,
                            size: Some(uniform_size.try_into().unwrap()),
                        }),
                    },
                ],
            });


            let shader_source = load_glsl(include_str!("render/back/back.vert"), ShaderStage::Vertex);
            let shader_vs_raw = wgpu::util::make_spirv_raw(&shader_source);
            let shader_vs = wgpu::ShaderModuleDescriptorSpirV {
                label: Some("back.vert.spv"),
                source: shader_vs_raw,
            };
            let vs_module = unsafe { device.create_shader_module_spirv(&shader_vs) };

            let shader_source = load_glsl(include_str!("render/back/back.frag"), ShaderStage::Fragment);
            let shader_fs_raw = wgpu::util::make_spirv_raw(&shader_source);
            let shader_fs = wgpu::ShaderModuleDescriptorSpirV {
                label: Some("back.frag.spv"),
                source: shader_fs_raw,
            };
            let fs_module = unsafe { device.create_shader_module_spirv(&shader_fs) };

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Render Pipeline"),
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: StencilState {
                        front: StencilFaceState::IGNORE,
                        back: StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: wgpu::DepthBiasState {
                        constant: 0,
                        slope_scale: 0.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

            Pass {
                pipeline,
                bind_group,
                uniform_buf,
            }
        };

        let forward_pass = {
            let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &local_bind_group_layout],
                push_constant_ranges: &[],
            });
            
            let mx_total = generate_matrix(config.width as f32 / config.height as f32);
            let forward_uniforms = ForwardUniforms {
                proj: mx_total.into(),
                num_lights: [lights.len() as u32, 0, 0, 0],
            };
            let uniform_size = std::mem::size_of::<ForwardUniforms>() as wgpu::BufferAddress;
            let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Forward Uniform Buffer"),
                contents: cast_slice(&[forward_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform_buf,
                            offset: 0,
                            size: Some(uniform_size.try_into().unwrap()),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &light_uniform_buf,
                            offset: 0,
                            size: Some(light_uniform_size.try_into().unwrap()),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&shadow_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                    },
                ],
            });

            let shader_source = load_glsl(include_str!("render/forward/forward.vert"), ShaderStage::Vertex);
            let shader_vs_raw = wgpu::util::make_spirv_raw(&shader_source);
            let shader_vs = wgpu::ShaderModuleDescriptorSpirV {
                label: Some("forward.vert.spv"),
                source: shader_vs_raw,
            };
            let vs_module = unsafe { device.create_shader_module_spirv(&shader_vs) };
            
            let shader_source = load_glsl(include_str!("render/forward/forward.frag"), ShaderStage::Fragment);
            let shader_fs_raw = wgpu::util::make_spirv_raw(&shader_source);
            let shader_fs = wgpu::ShaderModuleDescriptorSpirV {
                label: Some("forward.frag.spv"),
                source: shader_fs_raw,
            };
            let fs_module = unsafe { device.create_shader_module_spirv(&shader_fs) };
        };
    }

    pub async fn run() {
        let mut close_requested = false;
        let mut view_update = false;

        let mut factor: f64 = 1.0;

        let event_loop = EventLoop::new().unwrap();

        let (window, scale_factor, size, surface, adapter, device, queue, mut config) = {
            let builder = WindowBuilder::new();
            let window = Arc::new(builder
                .with_title("Hello Wgpu!")
                .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
                .with_resizable(false)
                .with_maximized(false)
                .build(&event_loop).unwrap());
            let size = window.inner_size();
            let scale_factor = window.scale_factor();

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

            let mut features = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM | wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
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
            let selected_format = TextureFormat::Bgra8UnormSrgb;
            let swapchain_format = swapchain_capabilities
                .formats
                .iter()
                .find(|d| **d == selected_format)
                .expect("failed to select proper surface texture format!");

            let mut config = SurfaceConfiguration {
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

            (window, scale_factor, size, surface, adapter, device, queue, config)
        };

        Application::init(&config, &device);

        let mut gui_renderer = create_egui(&window, &device, &config);

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => {
                    gui_renderer.handle_input(&window, &event);

                    match event {
                        WindowEvent::CloseRequested => {
                            close_requested = true;
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
                                    let screen_descriptor = ScreenDescriptor {
                                        size_in_pixels: [config.width, config.height],
                                        pixels_per_point: window.scale_factor() as f32,
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
                                                            factor
                                                        ));
                                                        if ui.button("-").clicked() {
                                                            factor = (factor - 0.1).max(0.3);
                                                            println!("scale_factor: {}", factor);
                                                        }
                                                        if ui.button("+").clicked() {
                                                            factor = (factor + 0.1).min(3.0);
                                                            println!("scale_factor: {}", factor);
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
}


fn create_egui(window: &Window, device: &Device, config: &SurfaceConfiguration) -> GuiRenderer {
    let mut gui_renderer = GuiRenderer::new(&device, config.format, None, 1, &window);
    gui_renderer
}