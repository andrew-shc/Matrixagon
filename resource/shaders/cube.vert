#version 450

layout(location = 0) in vec3 pos;  // position
layout(location = 2) in uint txtr;  // texture coordinates

layout(location = 0) out uint pass_ind;  // texture index
layout(location = 1) out vec2 pass_txtr;  // texture coordinates
layout(location = 2) out uint pass_light;  // texture light (brightness) level

layout(set = 1, binding = 0) uniform MVP {
    mat4 proj;
    mat4 view;
    mat4 world;
} matrix;

void main() {
    gl_Position = matrix.proj * matrix.view * matrix.world * vec4(pos, 1.0);
    // texture index shares the same variable as the txtr attribute
    pass_ind = txtr >> 16;
    // the texture has to be a vector for interpolation to work
    pass_txtr = vec2((txtr & 2u) >> 1u, txtr & 1u);

    pass_light = (txtr >> 2u) & 15u;
}
