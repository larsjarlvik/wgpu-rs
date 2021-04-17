#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in vec2 v_tex_coords;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() {
   vec4 base_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);

   if (base_color.a >= 0.3) {
      float depth = v_position.z / v_position.w;
      float dx = dFdx(depth);
      float dy = dFdx(depth);
      const float bias = max(abs(dx), abs(dy)) * 4.0;

      gl_FragDepth = depth + bias;
   } else {
      gl_FragDepth = 1.0;
   }
}
