#version 450

layout(location = 0) in vec3 position;
layout(set = 0, binding = 0) uniform Uniform { mat4 view_projection; };

void main() {
    gl_Position = view_projection * vec4(position, 1.0);
}
