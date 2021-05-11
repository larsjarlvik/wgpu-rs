#include "camera.glsl"

vec4 with_fog(vec4 color, vec3 position, float fade_distance) {
    float fog = smoothstep(cam.z_far / 4.0 * fade_distance, cam.z_far * fade_distance, distance(position, cam.eye_pos));
    return mix(color, vec4(0.312, 0.573, 0.757, 1.0), fog);
}

