#version 450

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_position;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform sampler t_sampler;

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    vec4 depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0);
    vec4 position = texelFetch(sampler2D(t_position, t_sampler), fragCoord, 0);
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    f_color = base_color;
}
