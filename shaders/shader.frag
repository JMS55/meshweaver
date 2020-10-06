#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(set = 2, binding = 0) uniform Light { vec3 light_position; };

layout(location = 0) out vec4 color;

void main() {
    vec3 object_color = normalize(position);

    vec3 light_color = vec3(1.0, 1.0, 1.0);

    vec3 ambient_color = light_color * 0.1;

    vec3 light_direction = normalize(light_position - position);
    float diffuse_strength = max(dot(normalize(normal), light_direction), 0.0);
    vec3 diffuse_color = light_color * diffuse_strength;

    color = vec4((ambient_color + diffuse_color) * object_color, 1.0);
}
