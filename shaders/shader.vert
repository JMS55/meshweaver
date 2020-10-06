#version 450

layout(location = 0) in vec3 position_in;
layout(location = 1) in vec3 normal_in;
layout(set = 0, binding = 0) uniform Camera { mat4 view_projection; };
layout(set = 1, binding = 0) uniform Transform { mat4 transform; };

layout(location = 0) out vec3 position_out;
layout(location = 1) out vec3 normal_out;

void main() {
    gl_Position = view_projection * transform * vec4(position_in, 1.0);

    position_out = position_in;
    normal_out = mat3(transpose(inverse(transform))) * normal_in;
}
