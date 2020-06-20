#version 450

layout(location = 0) in vec3 pos;  // position
layout(location = 1) in int ind;  // texture index
layout(location = 2) in uvec2 txtr; // texture coordinates

layout(location = 0) out int pass_ind;
layout(location = 1) out vec2 pass_txtr; // texture coordinates

layout(set = 1, binding = 0) uniform MVP {
    mat4 proj;
    mat4 view;
    mat4 world;
} matrix;

void main() {
    gl_Position = matrix.proj * matrix.view * matrix.world * vec4(pos, 1.0);
    pass_ind = ind;
    pass_txtr = vec2(txtr);
}
