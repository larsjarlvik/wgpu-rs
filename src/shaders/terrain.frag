#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in vec3 v_normals;

layout(location=0) out vec4 f_position;
layout(location=1) out vec4 f_normals;
layout(location=2) out vec4 f_base_color;

void main() {
   f_position = v_position;
   f_normals = vec4(v_normals, 1.0);
   f_base_color = vec4(0.4, 0.6, 0.07, 1.0);
}
