#version 450
#extension GL_GOOGLE_include_directive : require
#define BIAS 0.005

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform texture2D t_shadow;
layout(set = 0, binding = 4) uniform sampler t_sampler;

layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
    mat4 u_shadow_matrix;
};

layout(set=2, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

#include "include/light.glsl"

vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos = vec4(vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0, depth, 1.0);
    return (inverse(view_proj) * pos) / pos.w;
}

float get_shadow_factor(vec4 position) {
    vec4 texture_coords = u_shadow_matrix * position * 0.5 + 0.5;

    float shadowFactor = clamp(min(
        1.0 - abs(texture_coords.x),
        1.0 - abs(texture_coords.y)
    ) * 4.0, 0.0, 1.0);

    if(shadowFactor == 0.0) {
        return 1.0;
    }

    float offset = 1.0 / 8096.0; // TODO: uniform
    float total = 0.0;

    for (float y = -1.0; y <= 1.0 ; y++) {
        for (float x = -1.0; x <= 1.0 ; x++) {
            vec3 uvc = vec3(texture_coords.x + x * offset, texture_coords.y + y * offset, texture_coords.z + BIAS);
            total += texture(sampler2DShadow(t_shadow, t_sampler), uvc);
        }
    }

    return 0.5 + (total / 18.0);
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;

    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_view_proj);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 color;
    if (depth < 1.0) {
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz), 16.0, 1.0);
        color *= get_shadow_factor(position);
    }

    f_color = vec4(color, 1.0);
    gl_FragDepth = depth;
}
