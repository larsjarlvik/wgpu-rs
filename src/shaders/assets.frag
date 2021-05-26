#version 450

#define CAMERA_SET 1
#define LIGHT_SET 2
#define LIGHT_TEXTURE_SET 3

#include "include/camera.glsl"
#include "include/light.glsl"
#include "include/fog.glsl"

layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec4 v_position;
layout(location=2) in mat3 v_tangent;

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform sampler s_texture;

void main() {
    vec4 base_color = texture(sampler2D(t_diffuse, s_texture), v_tex_coords);
    if (base_color.a < 0.3) {
        discard; return;
    }

    vec4 normal = texture(sampler2D(t_normal, s_texture), v_tex_coords);
    vec3 n = normalize(v_tangent * (2.0 * normal.xyz - 1.0));

    f_color = vec4(light_shadow(v_position.xyz, n, base_color.rgb), 1.0);
    f_color = with_fog(f_color, v_position.xyz, 0.5);
}
