#version 450

layout(location = 0) in vec2 pos; // position
layout(location = 1) in vec4 col; // color

layout(location = 0) out vec4 pass_col;

void main() {
    gl_Position = vec4(vec3(pos, 0.0), 1.0);
    pass_col = col;
}
