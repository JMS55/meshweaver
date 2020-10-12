#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(set = 1, binding = 1) uniform texture2D mesh_texture;
layout(set = 1, binding = 2) uniform sampler texture_sampler;
layout(set = 2, binding = 0) uniform Light { vec3 light_position; };

layout(location = 0) out vec4 color;

void main() {
    vec4 object_color = texture(sampler2D(mesh_texture, texture_sampler), uv);

    vec3 light_color = vec3(1.0, 1.0, 1.0);

    vec3 ambient_color = light_color * 0.1;

    vec3 light_direction = normalize(light_position - position);
    float diffuse_strength = max(dot(normalize(normal), light_direction), 0.0);
    vec3 diffuse_color = light_color * diffuse_strength;

    color = vec4((ambient_color + diffuse_color) * object_color.rgb, object_color.a);
}
