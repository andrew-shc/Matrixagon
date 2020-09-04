#version 450

layout(location = 0) in vec4 pass_col; // color

layout(location = 0) out vec4 color; // color

void main() {
    color = pass_col;
}
