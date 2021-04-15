#version 450
#extension GL_GOOGLE_include_directive : require
#define CASCADE_COUNT 4

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

float get_shadow_factor(vec4 position, int texture_index) {
    vec4 homogeneous_coords = u_shadow_matrix[texture_index] * position;
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    const vec2 flip_correction = vec2(0.5, -0.5);
    vec3 light_local = vec3(
        homogeneous_coords.xy * flip_correction / homogeneous_coords.w + 0.5,
        homogeneous_coords.z / homogeneous_coords.w
    );

    return texture(sampler2DShadow(t_shadow[texture_index], t_shadow_sampler), light_local);
}

float get_shadow(vec4 position, float depth) {
    for(int i = 0; i < CASCADE_COUNT; i ++) {
        if (depth < u_shadow_split[i].x) {
            return get_shadow_factor(position, i);
        }
    }
}

vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos = vec4(vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0, depth, 1.0);
    return (inverse(view_proj) * pos) / pos.w;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;

    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_view_proj);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 color;
    if (depth < 1.0) {
        float shadow_factor = get_shadow(position, linearize_depth(depth));
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz), 16.0, 1.0, shadow_factor);
    }

    f_color = vec4(color, 1.0);
    gl_FragDepth = depth;
}
