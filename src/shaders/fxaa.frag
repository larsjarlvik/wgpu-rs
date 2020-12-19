#version 450
precision highp float;

#define FXAA_REDUCE_MIN (1.0 / 128.0)
#define FXAA_REDUCE_MUL (1.0 / 8.0)
#define FXAA_SPAN_MAX 8.0

layout(set = 0, binding = 0) uniform texture2D t_texture;
layout(set = 0, binding = 1) uniform sampler t_sampler;

layout(location=0) in vec2 resolution;
layout(location=1) in vec2 rgb_nw;
layout(location=2) in vec2 rgb_ne;
layout(location=3) in vec2 rgb_sw;
layout(location=4) in vec2 rgb_se;
layout(location=5) in vec2 rgb_m;
layout(location=6) in vec2 frag_coord;

layout(location=0) out vec4 f_color;

void main(void) {
    vec4 color;
    vec2 inverse_vp = vec2(1.0 / resolution.x, 1.0 / resolution.y);
    vec3 rgb_nw = texture(sampler2D(t_texture, t_sampler), rgb_nw).xyz;
    vec3 rgb_ne = texture(sampler2D(t_texture, t_sampler), rgb_ne).xyz;
    vec3 rgb_sw = texture(sampler2D(t_texture, t_sampler), rgb_sw).xyz;
    vec3 rgb_se = texture(sampler2D(t_texture, t_sampler), rgb_se).xyz;
    vec4 tex_color = texture(sampler2D(t_texture, t_sampler), rgb_m);
    vec3 rgb_m = tex_color.xyz;
    vec3 luma = vec3(0.299, 0.587, 0.114);
    float luma_nw = dot(rgb_nw, luma);
    float luma_ne = dot(rgb_ne, luma);
    float luma_sw = dot(rgb_sw, luma);
    float luma_se = dot(rgb_se, luma);
    float luma_m = dot(rgb_m,  luma);
    float luma_min = min(luma_m, min(min(luma_nw, luma_ne), min(luma_sw, luma_se)));
    float luma_max = max(luma_m, max(max(luma_nw, luma_ne), max(luma_sw, luma_se)));

    vec2 dir;
    dir.x = -((luma_nw + luma_ne) - (luma_sw + luma_se));
    dir.y =  ((luma_nw + luma_sw) - (luma_ne + luma_se));

    float dir_reduce = max((luma_nw + luma_ne + luma_sw + luma_se) * (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    float rcp_dir_min = 1.0 / (min(abs(dir.x), abs(dir.y)) + dir_reduce);

    dir = min(
        vec2(FXAA_SPAN_MAX, FXAA_SPAN_MAX),
        max(vec2(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX),
        dir * rcp_dir_min
    )) * inverse_vp;

    vec3 rgb_a = 0.5 * (
        texture(sampler2D(t_texture, t_sampler), frag_coord.xy * inverse_vp + dir * (1.0 / 3.0 - 0.5)).xyz +
        texture(sampler2D(t_texture, t_sampler), frag_coord.xy * inverse_vp + dir * (2.0 / 3.0 - 0.5)).xyz);
    vec3 rgb_b = rgb_a * 0.5 + 0.25 * (
        texture(sampler2D(t_texture, t_sampler), frag_coord.xy * inverse_vp + dir * -0.5).xyz +
        texture(sampler2D(t_texture, t_sampler), frag_coord.xy * inverse_vp + dir *  0.5).xyz);

    float luma_b = dot(rgb_b, luma);
    if ((luma_b < luma_min) || (luma_b > luma_max))
        color = vec4(rgb_a, tex_color.a);
    else
        color = vec4(rgb_b, tex_color.a);

    f_color = color;
}
