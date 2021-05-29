#version 450
#define CAMERA_SET 2
#define NOISE_SET 3
#include "include/camera.glsl"
#include "include/noise.glsl"
#include "include/environment.glsl"

layout(set=0, binding=0) uniform Uniforms {
    float wind_factor;
    float render_distance;
} uniforms;

layout(location=0) in vec3 a_position;
layout(location=3) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out float v_fade;

void main() {
    vec4 position = model_matrix * vec4(a_position, 1.0);
    float fade_out = cam.z_far * uniforms.render_distance;
    float dist = distance(position.xyz, cam.eye_pos);

    position.xyz += vec3(noise(position.xz + env.time * 0.001)) * uniforms.wind_factor * clamp(a_position.y * 0.2, 0.1, 1.0);

    v_fade = dist / fade_out;
    v_tex_coords = a_tex_coords;


    gl_Position = cam.view_proj * position;

    inverse(model_matrix); // TODO: Why is this needed? Get error if I remove it
}
