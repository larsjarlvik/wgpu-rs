#version 450

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;
layout(location=4) in vec3 v_normal;

layout(location=0) out vec4 f_color;

layout(set = 3, binding = 0) uniform texture2D t_depth_texture;
layout(set = 3, binding = 1) uniform texture2D t_refraction;
layout(set = 3, binding = 2) uniform texture2D t_reflection;
layout(set = 3, binding = 3) uniform sampler t_sampler;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

float water_depth(float depth) {
    float floor_distance = 2.0 * z_near * z_far / (z_far + z_near - (2.0 * depth - 1.0) * (z_far - z_near));
    float water_distance = 2.0 * z_near * z_far / (z_far + z_near - (2.0 * gl_FragCoord.z - 1.0) * (z_far - z_near));
    return floor_distance - water_distance;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;
    vec4 refraction = texelFetch(sampler2D(t_refraction, t_sampler), fragCoord, 0);
    vec4 reflection = texelFetch(sampler2D(t_reflection, t_sampler), ivec2(u_viewport_size.x - fragCoord.x, fragCoord.y), 0);

    float water_depth = water_depth(depth);
    vec3 water_color = mix(refraction.rgb, reflection.rgb, clamp(water_depth / 10.0 + 0.3, 0.1, 1.0));

    f_color = vec4(water_color, 1.0);
}
