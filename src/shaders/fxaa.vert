#version 450

layout(set=1, binding=0) uniform Uniforms {
    vec2 a_resolution;
};

layout(location=0) out vec2 resolution;
layout(location=1) out vec2 rgb_nw;
layout(location=2) out vec2 rgb_ne;
layout(location=3) out vec2 rgb_sw;
layout(location=4) out vec2 rgb_se;
layout(location=5) out vec2 rgb_m;
layout(location=6) out vec2 frag_coord;

const vec2 positions[6] = vec2[6](
    vec2( 1.0, 1.0),
    vec2(-1.0, 1.0),
    vec2(-1.0,-1.0),
    vec2(-1.0,-1.0),
    vec2( 1.0,-1.0),
    vec2( 1.0, 1.0)
);

void main() {
    frag_coord = vec2(positions[gl_VertexIndex].x, -positions[gl_VertexIndex].y);
    frag_coord = (frag_coord + 1.0) * 0.5 * a_resolution;
    vec2 inverse_vp = 1.0 / a_resolution.xy;

    rgb_nw = (frag_coord + vec2(-1.0,-1.0)) * inverse_vp;
    rgb_ne = (frag_coord + vec2( 1.0,-1.0)) * inverse_vp;
    rgb_sw = (frag_coord + vec2(-1.0, 1.0)) * inverse_vp;
    rgb_se = (frag_coord + vec2( 1.0, 1.0)) * inverse_vp;
    rgb_m = vec2(frag_coord * inverse_vp);
    resolution = a_resolution;

    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
