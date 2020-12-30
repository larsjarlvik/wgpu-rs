#version 450
#extension GL_GOOGLE_include_directive : require
#include "include/elevation.glsl"

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normals;
layout(location=2) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(set=1, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec2 u_viewport_size;
};

layout(location=0) out vec4 v_position;
layout(location=1) out vec3 v_normals;
layout(location=2) out vec2 v_tex_coords;

void main() {
    v_position = model_matrix * vec4(a_position, 1.0);
    v_position.y += get_elevation(vec2(model_matrix[3][0], model_matrix[3][2]));

    v_normals = mat3(transpose(inverse(model_matrix))) * a_normals;
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * v_position;
}
