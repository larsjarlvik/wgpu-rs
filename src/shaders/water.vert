#version 450
#extension GL_GOOGLE_include_directive : require

#define NOISE_SET 2
#include "include/noise.glsl"

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec4 v_position;
layout(location=1) out mat3 v_tbn;
layout(location=4) out vec3 v_normal;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec2 u_viewport_size;
};

layout(set=1, binding=0) uniform Uniforms {
    float u_time;
};

float get_wave(vec2 xz) {
    vec2 offset1 = vec2(1.0, 0.5) * u_time * 0.000932;
    vec2 offset2 = vec2(0.5, 1.0) * u_time * 0.001334;
    float h1 = noise(xz * 0.213 + offset1);
    float h2 = noise(xz * 3.231 + offset2);
    return (h1 + h2) * 0.5;
}

vec3 calc_normal() {
    return normalize(vec3(
        get_wave(vec2((v_position.x - 1.0), v_position.z)) - get_wave(vec2((v_position.x + 1.0), v_position.z)),
        1.0,
        get_wave(vec2(v_position.x, (v_position.z - 1.0))) - get_wave(vec2(v_position.x, (v_position.z + 1.0)))
    ));
}

void main() {
    vec3 bitangent = normalize(cross(vec3(0.0, 0.0, 1.0), a_normal));
    vec3 tangent = normalize(cross(a_normal, bitangent));

    v_position = vec4(a_position, 1.0);
    v_position.y = get_wave(v_position.xz);

    vec3 normal = calc_normal();
    v_tbn = mat3(tangent, bitangent, normal);
    v_normal = normal;
    gl_Position = u_view_proj * v_position;
}
