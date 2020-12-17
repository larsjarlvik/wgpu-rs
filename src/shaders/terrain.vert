#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normals;

layout(location=0) out vec4 v_position;
layout(location=1) out vec3 v_normals;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
};

void main() {
    v_position = vec4(a_position, 1.0);
    v_normals = a_normals;
    gl_Position = u_view_proj * v_position;
}
