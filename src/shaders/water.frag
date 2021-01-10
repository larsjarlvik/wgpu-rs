#version 450
#extension GL_GOOGLE_include_directive : require
#include "include/world_position.glsl"

layout(location=0) out vec4 f_normals;
layout(location=1) out vec4 f_base_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 1, binding = 0) uniform sampler t_sampler;

layout(set=2, binding=0) uniform Camera {
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

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;
    if (depth == 1.0) discard;

    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_view_proj);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    f_normals = normal;
    f_base_color = base_color;
    gl_FragDepth = depth;

    vec4 pos = position;
    if (pos.y < 0.0) {
        f_base_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}
