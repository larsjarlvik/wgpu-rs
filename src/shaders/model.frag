#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=1) in mat3 v_tangent;

layout(location=0) out vec4 f_normals;
layout(location=1) out vec4 f_base_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform sampler s_texture;

void main() {
   vec4 base_color = texture(sampler2D(t_diffuse, s_texture), v_tex_coords);
   vec4 normal = texture(sampler2D(t_normal, s_texture), v_tex_coords);

   if (base_color.a < 0.3) {
      discard; return;
   }

   f_normals = vec4(normalize(v_tangent * (normal.xyz * 2.0 - 1.0)), 1.0);
   f_base_color = base_color;
}
