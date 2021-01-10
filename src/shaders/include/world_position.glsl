
vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos;
    pos.xy = vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0;
    pos.z = depth;
    pos.w = 1.0;

    pos = inverse(view_proj) * pos;
    pos /= pos.w;
    return pos;
}
