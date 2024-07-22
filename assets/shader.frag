#version 450
layout(location = 0) out vec4 outColor;

in vec4 v_Color;

void main() {
    outColor = v_Color;
}
