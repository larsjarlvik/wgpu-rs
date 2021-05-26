#include "camera.glsl"

vec4 with_fog(vec4 color, vec3 position, float fade_distance) {
    float max_distance = sqrt((cam.z_far * cam.z_far) + (cam.z_far * cam.z_far));

    float furthest = max_distance * fade_distance;
    float dist = distance(position, cam.eye_pos);

    float fog = smoothstep(furthest * 0.4, furthest, dist);
    return mix(color, vec4(0.312, 0.573, 0.757, color.a * clamp(1.0 - ((dist - (furthest * 0.9)) / (furthest * 0.1)), 0.0, 1.0)), fog);
}

