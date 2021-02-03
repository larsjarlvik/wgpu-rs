#version 450
#extension GL_GOOGLE_include_directive : require

const vec3 sky_color = vec3(0.6, 0.8, 0.9);

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform sampler t_sampler;

layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
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

vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos = vec4(vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0, depth, 1.0);
    return (inverse(view_proj) * pos) / pos.w;
}

vec3 sky() {
    mat4 proj = u_view_proj;
    proj[3][0] = 0.0; proj[3][1] = 0.0; proj[3][2] = 0.0;
    vec3 ray_dir = normalize(world_pos_from_depth(1.0, gl_FragCoord.xy / u_viewport_size, proj).xyz);

    vec3 sun = pow(max(dot(ray_dir, normalize(-u_light_dir)), 0.0) * 0.993, 100.0) * vec3(1, 0.8, 0.3);
    float theta = atan(max(ray_dir.y, 0.0) / length(vec2(ray_dir.x, ray_dir.z)));
    float sky_factor = pow(abs(sin(theta)), 0.5);
    vec3 sky = sky_factor * sky_color + (1.0 - sky_factor) * vec3(1.0, 1.0, 0.9);
    return pow(sky + sun, vec3(2.2));
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;

    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_view_proj);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 color;
    if (depth < 1.0) {
        float fog = smoothstep(z_far / 4.0, z_far, linearize_depth(depth));
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz), 16.0, 1.0);
        color = mix(color, sky_color, fog);
    } else {
        color = sky();
    }

    f_color = vec4(color, 1.0);
    gl_FragDepth = depth;
}
