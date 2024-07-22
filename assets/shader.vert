#version 450

layout(location = 0) in vec2 a_Position;
layout(location = 1) in vec4 a_Color;

const vec2 positions[3] = vec2[3](
vec2(0.0, -0.5),
vec2(0.5, 0.5),
vec2(-0.5, 0.5)
);
out vec4 v_Color;
void main() {
    v_Color = a_Color;
    gl_Position = vec4(a_Position, 0.0, 1.0);
}
