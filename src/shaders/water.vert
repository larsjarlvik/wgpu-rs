#version 450

#define NOISE_SET 2
#include "include/noise.glsl"
#include "include/waves.glsl"

layout(location=0) in vec3 a_position;

layout(location=0) out vec4 v_position;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

void main() {
    v_position = vec4(a_position, 1.0);
    v_position.y = get_wave(v_position.xz);
    gl_Position = u_view_proj * v_position;
}
