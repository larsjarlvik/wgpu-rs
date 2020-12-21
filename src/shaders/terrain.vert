#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_tangent;
layout(location=2) in vec3 a_bitangent;
layout(location=3) in vec3 a_normal;

layout(location=0) out vec4 v_position;
layout(location=1) out mat3 v_tbn;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
};

void main() {
    v_position = vec4(a_position, 1.0);
    v_tbn = mat3(a_tangent, a_bitangent, a_normal);
    gl_Position = u_view_proj * v_position;
}
