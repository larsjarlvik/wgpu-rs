#version 450

layout(location=0) in vec2 v_tex_coords;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 2) uniform sampler s_texture;

void main() {
   vec4 base_color = texture(sampler2D(t_diffuse, s_texture), v_tex_coords);
   if (base_color.a < 0.3) {
      discard;
   }
}
