#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in vec2 v_tex_coords;

layout(location=0) out vec4 f_position;
layout(location=1) out vec4 f_base_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() {
   vec4 base_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
   if (base_color.a < 0.3) {
      discard; return;
   }

   f_position = v_position;
   f_base_color = base_color;
}
