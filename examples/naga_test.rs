use glsl_naga::utils::*;
fn main() {
    let glsl = include_str!("../assets/glsl-in/shader.frag");

    let code = glsl_to_wgsl(&glsl, naga::ShaderStage::Fragment);

    println!("{}", code);
}