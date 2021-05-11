#version 450
#define CAMERA_SET 1
#include "include/camera.glsl"

layout(location=0) in vec3 a_position;
layout(location=3) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(location=0) out vec2 v_tex_coords;

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = cam.view_proj * model_matrix * vec4(a_position, 1.0);

    inverse(model_matrix); // TODO: Why is this needed? Get error if I remove it
}
