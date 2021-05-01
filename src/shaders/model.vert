#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normals;
layout(location=2) in vec4 a_tangents;
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
layout(location=1) out mat3 v_tangent;

void main() {
    vec4 t = normalize(a_tangents);
    vec3 normal_w = normalize(vec3(model_matrix * vec4(a_normals, 0.0)));
    vec3 tangent_w = normalize(vec3(model_matrix * a_tangents));
    vec3 bitangent_w = cross(normal_w, tangent_w) * t.w;

    v_tangent = mat3(tangent_w, bitangent_w, normal_w);
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * model_matrix * vec4(a_position, 1.0);

    inverse(model_matrix); // TODO: Why is this needed? Get error if I remove it
}
