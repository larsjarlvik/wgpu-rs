#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normals;
layout(location=2) in vec2 a_tex_coords;
layout(location=5) in mat4 model_matrix;

layout(set=1, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
};

layout(location=0) out vec4 v_position;
layout(location=1) out vec3 v_normals;
layout(location=2) out vec2 v_tex_coords;

void main() {
    v_position = model_matrix * vec4(a_position, 1.0);
    v_normals = mat3(transpose(inverse(model_matrix))) * a_normals;
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * v_position;
}
