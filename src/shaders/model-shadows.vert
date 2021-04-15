#version 450

layout(location=0) in vec3 a_position;
layout(location=2) in vec2 a_tex_coords;
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

layout(location=0) out vec4 v_position;
layout(location=1) out vec2 v_tex_coords;

void main() {
    mat3(transpose(inverse(model_matrix))); // TODO: Why is this needed?
    v_tex_coords = a_tex_coords;
    v_position = u_view_proj * model_matrix * vec4(a_position, 1.0);
    gl_Position = v_position;
}
