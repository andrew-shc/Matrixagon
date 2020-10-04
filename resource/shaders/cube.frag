#version 450

layout(location = 0) flat in uint pass_ind;
layout(location = 1)      in vec2 pass_txtr;
layout(location = 2) flat in uint pass_light;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2DArray txtr;

void main() {
    /*
    texture index

    0 - 0b00
    1 - 0b01
    2 - 0b10
    3 - 0b11

    1 -- 3
    | \  |
    |  \ |
    0 -- 2
    */

    f_color = texture(txtr, vec3(pass_txtr, pass_ind)) * vec4(float(pass_light)/15., float(pass_light)/15., float(pass_light)/15., 1.0);
}
