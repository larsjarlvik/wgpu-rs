#version 450

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_position;
layout(set = 0, binding = 2) uniform texture2D t_normal;
layout(set = 0, binding = 3) uniform texture2D t_base_color;
layout(set = 0, binding = 4) uniform sampler t_sampler;

layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
};

layout(set=2, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
};

vec3 calculate_light(vec3 position, vec3 normal) {
    vec3 ambient_color = u_light_color * u_ambient_strength;
    vec3 inverse_light_dir = -u_light_dir;

    float diffuse_strength = max(dot(normal, inverse_light_dir), 0.0);
    vec3 diffuse_color = u_light_color * diffuse_strength;

    vec3 view_dir = normalize(u_eye_pos - position);
    vec3 half_dir = normalize(view_dir + inverse_light_dir);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 16);
    vec3 specular_color = specular_strength * u_light_color;

    return ambient_color + diffuse_color + specular_color;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    vec4 depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0);
    if (depth.x == 1.0) {
        discard; return;
    }

    vec4 position = texelFetch(sampler2D(t_position, t_sampler), fragCoord, 0);
    vec4 normal = texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0);
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    f_color = vec4(base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz)), base_color.a);
}
