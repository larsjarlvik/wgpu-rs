#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;

layout(location=0) out vec4 f_position;
layout(location=1) out vec4 f_normal;
layout(location=2) out vec4 f_base_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse[2];
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
   vec3 normal = texture(sampler2D(t_diffuse[1], s_diffuse), v_position.xz * 0.25).xyz;
   normal = normalize(v_tbn * (normal * 2.0 - 1.0));

   f_position = v_position;
   f_normal = vec4(normal, 1.0);
   f_base_color = texture(sampler2D(t_diffuse[0], s_diffuse), f_position.xz * 0.25);
}
