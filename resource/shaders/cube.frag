#version 450

layout(location = 0) flat in uint pass_ind;
layout(location = 1) in vec2 pass_txtr;

layout(location = 0) out vec4 f_color;

// TODO: Problem with the texture length; we will have to manually set the length for now

// layout(set = 0, binding = 0) uniform sampler2Darray txtr;

layout(set = 0, binding = 0) uniform sampler2D txtr[6];  // Maximum texture length of 64

void main() {
    /*
    0 - 0b00
    1 - 0b01
    2 - 0b10
    3 - 0b11

    1 -- 3
    | \  |
    |  \ |
    0 -- 2
    */

    f_color = texture(txtr[pass_ind >> 16], pass_txtr);
}
