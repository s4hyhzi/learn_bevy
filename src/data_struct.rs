use std::ops::Range;
use std::rc::Rc;
use wgpu::Buffer;

#[allow(dead_code)]
#[derive(Debug)]
struct CubeDesc {
    offset: cgmath::Vector3<f32>,
    angle: f32,
    scale: f32,
    rotation: f32,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Entity {
    pub mx_world: cgmath::Matrix4<f32>,
    pub rotation_speed: f32,
    pub color: wgpu::Color,
    pub vertex_buf: Rc<wgpu::Buffer>,
    pub index_buf: Rc<wgpu::Buffer>,
    pub index_count: usize,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Light {
    pub pos: cgmath::Point3<f32>,
    pub color: wgpu::Color,
    pub fov: f32,
    pub depth: Range<f32>,
    pub target_view: wgpu::TextureView,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Pass {
    pub pipeline: wgpu::RenderPipeline,
    // pub bind_group: wgpu::BindGroup,
    // pub uniform_buf: wgpu::Buffer,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct State {
    // light:Vec<Light>,
    // lights_are_dirty: bool,
    // light_uniform_buf: wgpu::Buffer,
    // entity: Vec<Entity>,
    // shadow_pass: Pass,
    // forward_depth: wgpu::TextureView,
    pub forward_pass: Pass,
}