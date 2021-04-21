#version 450
#define CASCADE_COUNT 4
#define OFFSET 1.2

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform texture2D t_shadow[3];
layout(set = 0, binding = 4) uniform sampler t_sampler;
layout(set = 0, binding = 5) uniform samplerShadow t_shadow_sampler;

layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
    mat4 u_shadow_matrix[CASCADE_COUNT];
    vec4 u_shadow_split[CASCADE_COUNT];
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

float linearize_depth(float d) {
    return z_near * z_far / (z_far + d * (z_near - z_far));
}

float get_shadow_factor(vec3 position, int cascade_index) {
    vec4 shadow_coords = u_shadow_matrix[cascade_index] * vec4(position, 1.0);
    if (shadow_coords.w <= 0.0) {
        return 0.0;
    }

    const vec2 flip_correction = vec2(0.5, -0.5);
    vec3 light_local = vec3(
        shadow_coords.xy * flip_correction / shadow_coords.w + 0.5,
        shadow_coords.z / shadow_coords.w
    );

    vec2 texel_size = 1.0 / textureSize(sampler2DShadow(t_shadow[cascade_index], t_shadow_sampler), 0);
    float total = 0.0;

    for (float y = -OFFSET; y <= OFFSET; y += OFFSET) {
        for (float x = -OFFSET; x <= OFFSET; x += OFFSET) {
            vec2 offset = vec2(x, y) * texel_size;
            vec3 uvc = vec3(light_local.xy + offset, light_local.z);
            total += texture(sampler2DShadow(t_shadow[cascade_index], t_shadow_sampler), uvc);
        }
    }

    return total / 9.0;
}

float get_shadow(vec3 position) {
    for(int i = 0; i < CASCADE_COUNT; i ++) {
        if (distance(u_eye_pos, position) < u_shadow_split[i].x) {
            return get_shadow_factor(position, i);
        }
    }
    return 1.0;
}

vec3 world_pos_from_depth(float depth, vec2 uv, mat4 inv_matrix) {
    vec3 ndc = vec3(vec2(uv.x, 1.0 - uv.y) * 2.0 - 1.0, depth);
    vec4 p = inv_matrix * vec4(ndc, 1.0);
    return p.xyz / p.w;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;

    vec3 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, inverse(u_view_proj));
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 color;
    if (depth < 1.0) {
        float shadow_factor = get_shadow(position);
        color = base_color.rgb * calculate_light(position, normalize(normal.xyz), 16.0, 1.0, shadow_factor);
    }

    f_color = vec4(color, 1.0);
    gl_FragDepth = depth;
}
