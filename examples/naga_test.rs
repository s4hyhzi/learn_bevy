
fn main() {
    use naga::ShaderStage;
    use naga::front::glsl::{Frontend, Options};
    use naga::valid::{Capabilities, Validator};
    use naga::valid::ValidationFlags;
    use naga::back::wgsl;

    let glsl = r#"
    #version 450

    layout(location = 0) in ivec4 a_Pos;
    layout(location = 1) in ivec4 a_Normal;

    layout(location = 0) out vec3 v_Normal;
    layout(location = 1) out vec4 v_Position;

    layout(set = 0, binding = 0) uniform Globals {
        mat4 u_ViewProj;
        uvec4 u_NumLights;
    };
    layout(set = 1, binding = 0) uniform Entity {
        mat4 u_World;
        vec4 u_Color;
    };

    void main() {
        v_Normal = mat3(u_World) * vec3(a_Normal.xyz);
        v_Position = u_World * vec4(a_Pos);
        gl_Position = u_ViewProj * v_Position;
    }
"#;

    let mut frontend = Frontend::default();
    let options = Options::from(ShaderStage::Vertex);
    let Ok(res) = frontend.parse(&options, glsl) else { panic!("Failed to parse shader") };
    // println!("{:?}", res);
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let Ok(module_info) = validator.validate(&res) else { panic!("Failed to validate shader") };
    // println!("{:?}", module_info);
    let code = wgsl::write_string(&res, &module_info, wgsl::WriterFlags::all() ).unwrap();
    println!("{}", code);

}