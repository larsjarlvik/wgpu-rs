#version 450

layout(location=0) in vec3 a_position;
layout(location=3) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(set=1, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

layout(location=0) out vec2 v_tex_coords;

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * model_matrix * vec4(a_position, 1.0);

    inverse(model_matrix); // TODO: Why is this needed? Get error if I remove it
}
