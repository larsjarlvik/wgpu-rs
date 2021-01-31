#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;
layout(location=4) in vec3 v_normal;

layout(location=0) out vec4 f_normal;
layout(location=1) out vec4 f_base_color;

void main() {
    f_normal = vec4(v_normal, 1.0);
    f_base_color = vec4(0.5, 0.77, 0.87, 1.0);
}
