#version 450

#define MAX_AMPLITUDE 1.0

layout(location=0) out vec4 f_position;
layout(location=1) out vec4 f_normals;
layout(location=2) out vec4 f_base_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_position;
layout(set = 0, binding = 2) uniform texture2D t_normal;
layout(set = 0, binding = 3) uniform texture2D t_base_color;
layout(set = 1, binding = 0) uniform sampler t_sampler;

layout(set=2, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float u_z_near;
    vec3 u_look_at;
    float u_z_far;
    vec2 u_viewport_size;
    float u_enable_clip;
    float u_clip_y;
};

layout(set=3, binding=0) uniform Water {
    mat4 u_inverse_view_proj;
};

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;
    if (depth == 1.0) discard;

    vec4 position = texelFetch(sampler2D(t_position, t_sampler), fragCoord, 0);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    f_position = position;
    f_normals = normal;

    vec4 world_position = inverse(u_view_proj) * position;
    if (world_position.y < 0.0) {
        f_base_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        f_base_color = base_color;
    }
}
