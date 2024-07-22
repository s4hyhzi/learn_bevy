use glsl_naga::utils::*;
fn main() {
    let glsl = include_str!("../assets/shader.vert");

    let code = glsl_to_wgsl(&glsl, naga::ShaderStage::Vertex);

    println!("{}", code);
}