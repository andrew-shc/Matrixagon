#version 450

layout(location = 0) flat in int pass_ind;
layout(location = 1) in vec2 pass_txtr;

layout(location = 0) out vec4 f_color;

// TODO: Problem with the texture length; we will have to manually set the length for now
layout(set = 0, binding = 0) uniform sampler2D txtr[9];  // Maximum texture length of 64

void main() {
    f_color = texture(txtr[pass_ind], pass_txtr);
}
