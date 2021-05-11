#version 450

#define NOISE_SET 2
#include "include/noise.glsl"
#include "include/waves.glsl"
#include "include/camera.glsl"

layout(location=0) in vec2 a_position;
layout(location=0) out vec4 v_position;

layout(set=4, binding=0) uniform Node {
    vec2 u_translation;
};

void main() {
    vec2 pos = a_position + u_translation;
    v_position = vec4(pos.x, 0.0, pos.y, 1.0);
    v_position.y = get_wave(v_position.xz);
    gl_Position = cam.view_proj * v_position;
}

