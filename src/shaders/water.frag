#version 450
#extension GL_GOOGLE_include_directive : require

#define NOISE_SET 2
#include "include/noise.glsl"

#define SURFACE_COLOR vec3(0.0078, 0.4176, 0.6)
#define DEPTH_COLOR vec3(0.0039, 0.00196, 0.145)
#define EXTINCTION vec3(7.0, 30.0, 40.0)

layout(location=0) in vec4 v_position;

layout(location=0) out vec4 f_color;

layout(set = 3, binding = 0) uniform texture2D t_depth_texture;
layout(set = 3, binding = 1) uniform texture2D t_refraction;
layout(set = 3, binding = 2) uniform texture2D t_reflection;
layout(set = 3, binding = 3) uniform sampler t_sampler;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

// TODO: Uniforms
vec3 u_light_dir = vec3(0.5, -1.0, 0.0);
float u_ambient_strength = 0.3;
vec3 u_light_color = vec3(1.0, 0.9, 0.5);
float u_light_intensity = 1.0;

#include "include/light.glsl"

layout(set=1, binding=0) uniform Uniforms {
    float u_time;
};

float get_wave(vec2 xz) {
    vec2 offset1 = vec2(1.0, 0.5) * u_time * 0.000932;
    vec2 offset2 = vec2(0.5, 1.0) * u_time * 0.001334;
    float h1 = noise(xz * 0.213 + offset1);
    float h2 = noise(xz * 3.231 + offset2);
    return -(h1 + h2) * 0.5;
}

vec3 calc_normal() {
    vec3 pos = v_position.xyz * 0.5;
    return normalize(vec3(
        get_wave(vec2((pos.x - 1.0), pos.z)) - get_wave(vec2((pos.x + 1.0), pos.z)),
        1.0,
        get_wave(vec2(pos.x, (pos.z - 1.0))) - get_wave(vec2(pos.x, (pos.z + 1.0)))
    ));
}

float water_depth(float depth) {
    float floor_distance = z_near * z_far / (z_far + depth * (z_near - z_far));
    float water_distance = z_near * z_far / (z_far + gl_FragCoord.z * (z_near - z_far));
    return floor_distance - water_distance;
}

void main() {
    vec3 normal = calc_normal();
    vec2 fragCoord = vec2(gl_FragCoord.xy / u_viewport_size) + normal.xz * 0.01;

    float log_depth = texture(sampler2D(t_depth_texture, t_sampler), fragCoord).r;
    float depth = water_depth(log_depth) * 0.3;

    vec3 ground = texture(sampler2D(t_refraction, t_sampler), fragCoord).rgb;
    vec3 reflection = texture(sampler2D(t_reflection, t_sampler), vec2(1.0 - fragCoord.x, fragCoord.y)).rgb;
    vec3 refraction = mix(mix(ground, SURFACE_COLOR, clamp(depth / 2.0, 0.0, 1.0)), DEPTH_COLOR, clamp(depth / EXTINCTION, 0.0, 1.0));

    vec3 light = calculate_light(v_position.xyz, normal, 200.0, 2.0);
    vec3 view_dir = normalize(u_eye_pos - v_position.xyz);

    float fresnel = pow(dot(view_dir, vec3(0.0, 1.0, 0.0)), 1.2);
    vec3 water_color = mix(mix(reflection * light, SURFACE_COLOR, 0.3), refraction, clamp(fresnel, 0.25, 1.0));
    water_color = clamp(water_color, 0.0, 1.0);
    water_color = mix(ground, water_color, clamp(depth * 10.0, 0.0, 1.0));

    f_color = vec4(water_color, 1.0);
}
