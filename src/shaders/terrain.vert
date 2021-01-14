#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec4 v_position;
layout(location=1) out mat3 v_tbn;
layout(location=4) out vec3 v_normal;
layout(location=5) out vec2 v_color;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec2 u_viewport_size;
};

void main() {
    vec3 bitangent = normalize(cross(vec3(0.0, 0.0, 1.0), a_normal));
    vec3 tangent = normalize(cross(a_normal, bitangent));

    v_position = vec4(a_position, 1.0);
    v_tbn = mat3(tangent, bitangent, a_normal);
    v_normal = a_normal;
    v_color = vec2(mod(a_position.x / 40.0, 1.0), mod(a_position.z / 40.0, 1.0));
    gl_Position = u_view_proj * v_position;
}
