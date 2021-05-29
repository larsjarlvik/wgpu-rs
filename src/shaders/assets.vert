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
layout(location=1) in vec3 a_normals;
layout(location=2) in vec4 a_tangents;
layout(location=3) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec4 v_position;
layout(location=2) out mat3 v_tangent;

void main() {
    vec4 t = normalize(a_tangents);
    vec3 normal_w = normalize(vec3(model_matrix * vec4(a_normals, 0.0)));
    vec3 tangent_w = normalize(vec3(model_matrix * a_tangents));
    vec3 bitangent_w = cross(normal_w, tangent_w) * t.w;

    v_tangent = mat3(tangent_w, bitangent_w, normal_w);
    v_tex_coords = a_tex_coords;
    v_position = model_matrix * vec4(a_position, 1.0);
    v_position.xyz += vec3(noise(v_position.xz + env.time * 0.001)) * uniforms.wind_factor * clamp(a_position.y * 0.2, 0.1, 1.0);

    gl_Position = cam.view_proj * v_position;

    inverse(model_matrix); // TODO: Why is this needed? Get error if I remove it
}
