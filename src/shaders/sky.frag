#version 450

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_texture;
layout(set = 0, binding = 2) uniform sampler t_sampler;

layout(location=0) out vec4 f_color;

#include "include/camera.glsl"

layout(set=1, binding=0) uniform SkyUniforms {
    vec3 light_dir;
    float not_used;
    vec3 color;
    float fade_distance;
} sky;

float linearize_depth(float d) {
    return cam.z_near * cam.z_far / (cam.z_far + d * (cam.z_near - cam.z_far));
}

vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos = vec4(vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0, depth, 1.0);
    return (inverse(view_proj) * pos) / pos.w;
}

vec3 sky_color() {
    mat4 proj = cam.view_proj;
    proj[3][0] = 0.0; proj[3][1] = 0.0; proj[3][2] = 0.0;
    vec3 ray_dir = normalize(world_pos_from_depth(0.6, gl_FragCoord.xy / cam.viewport_size, proj).xyz);

    vec3 sun = pow(max(dot(ray_dir, normalize(-sky.light_dir)), 0.0) * 0.993, 50.0) * vec3(1, 0.7, 0.3);
    float theta = atan(max(ray_dir.y, 0.0) / length(vec2(ray_dir.x, ray_dir.z)));
    float sky_factor = pow(abs(sin(theta)), 0.5);
    vec3 sky = sky_factor * sky.color + (1.0 - sky_factor) * vec3(1.0, 1.0, 0.9);
    return pow(sky + sun, vec3(2.2));
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;
    vec4 color = texelFetch(sampler2D(t_texture, t_sampler), fragCoord, 0);

    if (depth < 1.0) {
        float fog = smoothstep(cam.z_far / 4.0 * sky.fade_distance, cam.z_far * sky.fade_distance, linearize_depth(depth));
        f_color = mix(color, vec4(sky.color, 1.0), fog);
    } else {
        f_color = vec4(sky_color(), 1.0);
    }
}

