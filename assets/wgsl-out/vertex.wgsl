struct Globals {
    u_ViewProj: mat4x4<f32>,
    u_NumLights: vec4<u32>,
}

struct Entity {
    u_World: mat4x4<f32>,
    u_Color: vec4<f32>,
}

struct VertexOutput {
    @location(0) v_Normal: vec3<f32>,
    @location(1) v_Position: vec4<f32>,
    @builtin(position) gl_Position: vec4<f32>,
}

var<private> a_Pos_1: vec4<i32>;
var<private> a_Normal_1: vec4<i32>;
var<private> v_Normal: vec3<f32>;
var<private> v_Position: vec4<f32>;
@group(0) @binding(0)
var<uniform> global: Globals;
@group(1) @binding(0)
var<uniform> global_1: Entity;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    let _e12: mat4x4<f32> = global_1.u_World;
    let _e22: vec4<i32> = a_Normal_1;
    v_Normal = (mat3x3<f32>(_e12[0].xyz, _e12[1].xyz, _e12[2].xyz) * vec3<f32>(_e22.xyz));
    let _e26: mat4x4<f32> = global_1.u_World;
    let _e27: vec4<i32> = a_Pos_1;
    v_Position = (_e26 * vec4<f32>(_e27));
    let _e31: mat4x4<f32> = global.u_ViewProj;
    let _e32: vec4<f32> = v_Position;
    gl_Position = (_e31 * _e32);
    return;
}

@vertex
fn main(@location(0) @interpolate(flat) a_Pos: vec4<i32>, @location(1) @interpolate(flat) a_Normal: vec4<i32>) -> VertexOutput {
    a_Pos_1 = a_Pos;
    a_Normal_1 = a_Normal;
    main_1();
    let _e21: vec3<f32> = v_Normal;
    let _e23: vec4<f32> = v_Position;
    let _e25: vec4<f32> = gl_Position;
    return VertexOutput(_e21, _e23, _e25);
}
