#version 450
#include "include/camera.glsl"

layout(location=0) out vec4 f_color;

layout(set=1, binding=0) uniform SkyUniforms {
    vec3 light_dir;
    float not_used;
    vec3 color;
    float fade_distance;
} sky;

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
    f_color = vec4(sky_color(), 1.0);
}

