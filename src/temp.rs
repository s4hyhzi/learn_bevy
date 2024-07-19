use std::io::Read;
pub use std::ops::Range;
pub use std::rc::Rc;

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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

pub struct Light {
    pub pos: cgmath::Point3<f32>,
    pub color: wgpu::Color,
    pub fov: f32,
    pub depth: Range<f32>,
    pub target_view: wgpu::TextureView,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightRaw {
    pub proj: [[f32; 4]; 4],
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl Light {
    pub fn to_raw(&self) -> LightRaw {
        use cgmath::{Deg, Matrix4, PerspectiveFov, Point3, Vector3};

        let mx_view = Matrix4::look_at_rh(self.pos, Point3::new(0.0, 0.0, 0.0), Vector3::unit_z());

        let projection = PerspectiveFov {
            fovy: Deg(self.fov).into(),
            aspect: 1.0,
            near: self.depth.start,
            far: self.depth.end,
        };

        let mx_view_proj = OPENGL_TO_WGPU_MATRIX *
            cgmath::Matrix4::from(projection.to_perspective()) * mx_view;

        LightRaw {
            proj: *mx_view_proj.as_ref(),
            pos: [self.pos.x, self.pos.y, self.pos.z, 1.0],
            color: [
                self.color.r as f32,
                self.color.g as f32,
                self.color.b as f32,
                1.0,
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ForwardUniforms {
    pub proj: [[f32; 4]; 4],
    pub num_lights: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EntityUniforms {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

#[repr(C)]
pub struct ShadowUniforms {
    pub proj: [[f32; 4]; 4],
}

pub struct Pass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
}

pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

pub fn spirv_to_u32_array(bytes: &[u8]) -> &[u32] {
    assert_eq!(bytes.len() % 4, 0);
    unsafe {
        std::slice::from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / 4)
    }
}

pub fn load_glsl(code: &str, stage: ShaderStage) -> Vec<u8> {
    let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
    };

    let Ok(mut spirv) = glsl_to_spirv::compile(code, ty)else { panic!("Failed to compile shader") };
    let mut buffer = vec![];
    spirv.read_to_end(&mut buffer).expect("Failed to read the file");
    buffer
}

/**
 * Generate a perspective matrix with a given aspect ratio.
 * @param {f32} aspect_ratio.
 */
pub fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 20.0);
    let mx_view = cgmath::Matrix4::look_at_rh(
        cgmath::Point3::new(3.0f32, -10.0, 6.0),
        cgmath::Point3::new(0f32, 0.0, 0.0),
        cgmath::Vector3::unit_z(),
    );
    OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view
}
