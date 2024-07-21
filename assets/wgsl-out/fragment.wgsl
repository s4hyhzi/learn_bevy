struct Light {
    proj: mat4x4<f32>,
    pos: vec4<f32>,
    color: vec4<f32>,
}

struct Globals {
    u_ViewProj: mat4x4<f32>,
    u_NumLights: vec4<u32>,
}

struct Lights {
    u_Lights: array<Light, 10>,
}

struct Entity {
    u_World: mat4x4<f32>,
    u_Color: vec4<f32>,
}

struct FragmentOutput {
    @location(0) o_Target: vec4<f32>,
}

const MAX_LIGHTS: i32 = 10i;

var<private> v_Normal_1: vec3<f32>;
var<private> v_Position_1: vec4<f32>;
var<private> o_Target: vec4<f32>;
@group(0) @binding(0)
var<uniform> global: Globals;
@group(0) @binding(1)
var<uniform> global_1: Lights;
@group(0) @binding(2)
var t_Shadow: texture_depth_2d_array;
@group(0) @binding(3)
var s_Shadow: sampler_comparison;
@group(1) @binding(0)
var<uniform> global_2: Entity;

fn fetch_shadow(light_id: i32, homogeneous_coords: vec4<f32>) -> f32 {
    var light_id_1: i32;
    var homogeneous_coords_1: vec4<f32>;
    var light_local: vec4<f32>;

    light_id_1 = light_id;
    homogeneous_coords_1 = homogeneous_coords;
    let _e20: vec4<f32> = homogeneous_coords_1;
    if (_e20.w <= 0f) {
        {
            return 1f;
        }
    }
    let _e25: vec4<f32> = homogeneous_coords_1;
    let _e27: vec4<f32> = homogeneous_coords_1;
    let _e36: vec2<f32> = (((_e25.xy / vec2(_e27.w)) + vec2(1f)) / vec2(2f));
    let _e37: i32 = light_id_1;
    let _e38: vec4<f32> = homogeneous_coords_1;
    let _e40: vec4<f32> = homogeneous_coords_1;
    light_local = vec4<f32>(_e36.x, _e36.y, f32(_e37), (_e38.z / _e40.w));
    let _e49: vec4<f32> = light_local;
    let _e54: f32 = textureSampleCompare(t_Shadow, s_Shadow, _e49.xy, i32(_e49.z), _e49.w);
    return _e54;
}

fn main_1() {
    var normal: vec3<f32>;
    var ambient: vec3<f32> = vec3<f32>(0.05f, 0.05f, 0.05f);
    var color: vec3<f32>;
    var i: i32 = 0i;
    var light: Light;
    var shadow: f32;
    var light_dir: vec3<f32>;
    var diffuse: f32;

    let _e17: vec3<f32> = v_Normal_1;
    normal = normalize(_e17);
    let _e25: vec3<f32> = ambient;
    color = _e25;
    loop {
        let _e29: i32 = i;
        let _e30: vec4<u32> = global.u_NumLights;
        let _e34: i32 = i;
        if !(((_e29 < i32(_e30.x)) && (_e34 < MAX_LIGHTS))) {
            break;
        }
        {
            let _e41: i32 = i;
            let _e43: Light = global_1.u_Lights[_e41];
            light = _e43;
            let _e46: Light = light;
            let _e48: vec4<f32> = v_Position_1;
            let _e50: i32 = i;
            let _e51: Light = light;
            let _e53: vec4<f32> = v_Position_1;
            let _e55: f32 = fetch_shadow(_e50, (_e51.proj * _e53));
            shadow = _e55;
            let _e57: Light = light;
            let _e60: vec4<f32> = v_Position_1;
            let _e63: Light = light;
            let _e66: vec4<f32> = v_Position_1;
            light_dir = normalize((_e63.pos.xyz - _e66.xyz));
            let _e74: vec3<f32> = normal;
            let _e75: vec3<f32> = light_dir;
            let _e80: vec3<f32> = normal;
            let _e81: vec3<f32> = light_dir;
            diffuse = max(0f, dot(_e80, _e81));
            let _e85: vec3<f32> = color;
            let _e86: f32 = shadow;
            let _e87: f32 = diffuse;
            let _e89: Light = light;
            color = (_e85 + ((_e86 * _e87) * _e89.color.xyz));
        }
        continuing {
            let _e38: i32 = i;
            i = (_e38 + 1i);
        }
    }
    let _e94: vec3<f32> = color;
    let _e100: vec4<f32> = global_2.u_Color;
    o_Target = (vec4<f32>(_e94.x, _e94.y, _e94.z, 1f) * _e100);
    return;
}

@fragment
fn main(@location(0) v_Normal: vec3<f32>, @location(1) v_Position: vec4<f32>) -> FragmentOutput {
    v_Normal_1 = v_Normal;
    v_Position_1 = v_Position;
    main_1();
    let _e27: vec4<f32> = o_Target;
    return FragmentOutput(_e27);
}
