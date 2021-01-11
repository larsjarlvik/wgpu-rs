#version 450
#extension GL_GOOGLE_include_directive : require
#include "include/world_position.glsl"

#define FADE_SPEED 0.3
#define DEPTH_COLOR vec3(0.0078, 0.5176, 0.7)
#define BIG_DEPTH_COLOR vec3(0.0039, 0.00196, 0.145)
#define VISIBILITY 4.0
#define EXTINCTION vec3(7.0, 30.0, 40.0)
#define SHORE_HARDNESS 4.0
#define MAX_AMPLITUDE 1.0
#define SCALE 0.05
#define WIND vec2(-0.3, 0.7)

const vec3 sky_color = vec3(0.6, 0.8, 0.9);

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform sampler t_sampler;

layout(set = 1, binding = 0) uniform texture2D t_water_normal;
layout(set = 1, binding = 1) uniform sampler t_water_sampler;

layout(set=2, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
    float u_time;
};

layout(set=3, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float u_z_near;
    vec3 u_look_at;
    float u_z_far;
    vec2 u_viewport_size;
};

float linearize_depth(float d) {
    return u_z_near * u_z_far / (u_z_far + d * (u_z_near - u_z_far));
}

vec3 sky(vec3 rayDir) {
    vec3 sun = pow(max(dot(rayDir, normalize(u_light_dir)), 0.0), 12.0) * vec3(1, 0.8, 0.3);
    float theta = atan(rayDir.y / length(vec2(rayDir.x, rayDir.z)));
    float sky_factor = pow(abs(sin(theta)), 0.5);
    vec3 sky = sky_factor * sky_color + (1.0 - sky_factor) * vec3(1.0, 1.0, 0.9);
    return pow(sky + sun, vec3(2.2));
}

vec3 calculate_light(vec3 position, vec3 normal) {
    vec3 ambient_color = u_light_color * u_ambient_strength;
    vec3 inverse_light_dir = -u_light_dir;

    float diffuse_strength = max(dot(normal, inverse_light_dir), 0.0);
    vec3 diffuse_color = u_light_color * diffuse_strength;

    vec3 view_dir = normalize(u_eye_pos - position);
    vec3 half_dir = normalize(view_dir + inverse_light_dir);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 16);
    vec3 specular_color = specular_strength * u_light_color;

    return ambient_color + (diffuse_color + specular_color) * u_light_intensity;
}

float random(vec2 st) {
    return fract(sin(dot(st, vec2(12.9898,78.233))) * 43758.5453123);
}

float fresnel_term(vec3 normal, vec3 eye_vec) {
    float angle = 1.0 - clamp(dot(normal, eye_vec), 0.0, 1.0);
    float fresnel = angle * angle;
    fresnel = fresnel * fresnel;
    fresnel = fresnel * angle;
    return clamp(fresnel * (1.0 - 0.5) + 0.5, 0.0, 1.0);
}

vec3 water(vec3 position, vec3 color, vec3 sky_color) {
    vec3 eye_vec = normalize(u_eye_pos - position);
    float camera_depth = u_eye_pos.y - position.y;

    float t = (0.0 - u_eye_pos.y) / eye_vec.y;
    vec3 surface_point = u_eye_pos + eye_vec * t;

    float level = 0.0;
    vec2 tex_coord;
    for(int i = 0; i < 10; ++i)
    {
        tex_coord = (surface_point.xz + eye_vec.xz * 0.1) * SCALE + u_time * 0.00005 * WIND;
        float bias = texture(sampler2D(t_water_normal, t_water_sampler), tex_coord).y - 0.5;

        bias *= 0.1f;
        level += bias * MAX_AMPLITUDE * 2.0;
        t = (level - u_eye_pos.y) / eye_vec.y;
        surface_point = u_eye_pos + eye_vec * t;
    }

    float water_depth = length(position - surface_point);
    float water_depth2 = surface_point.y - position.y;

    if (water_depth < 0.0 || water_depth2 < 0.0) return color;

    vec3 depth_n = vec3(water_depth * FADE_SPEED);
    vec3 water_col = vec3(clamp(length(u_light_color) / u_light_intensity, 0.0, 1.0));

    vec3 refraction = mix(mix(color, DEPTH_COLOR * water_col, clamp(depth_n / VISIBILITY, 0.0, 1.0)),
            BIG_DEPTH_COLOR * water_col, clamp(water_depth2 / EXTINCTION, 0.0, 1.0));

    vec3 normal = normalize(texture(sampler2D(t_water_normal, t_water_sampler), tex_coord).xyz * 2.0 - 1.0);

    vec3 out_color = mix(refraction, sky_color, pow(abs(dot(eye_vec, normal)), 5.0));
    out_color = clamp(out_color, 0.0, 1.0);
    out_color = mix(refraction, out_color, clamp(water_depth * SHORE_HARDNESS, 0.0, 1.0));
    return out_color;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);

    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;
    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_view_proj);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 cam_front = normalize(u_eye_pos - u_look_at);
    vec3 cam_right = cross(cam_front, vec3(0, 1, 0));
    vec3 cam_up = cross(cam_right, cam_front);
    vec2 uv = (2.0 * gl_FragCoord.xy - u_viewport_size) / u_viewport_size.y;
    vec3 ray_dir = normalize(cam_front + uv.x * cam_right + uv.y * cam_up);

    vec3 color;

    if (depth < 1.0) {
        float fog = smoothstep(u_z_far / 4.0, u_z_far, linearize_depth(depth));
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz));

        if (position.y <= MAX_AMPLITUDE) {
            color = water(position.xyz, color, sky(vec3(ray_dir.x, -ray_dir.y, ray_dir.z)));
        }

        color = mix(color, sky_color, fog);
    } else {
        color = sky(ray_dir);
    }


    f_color = vec4(color, 1.0);
}
