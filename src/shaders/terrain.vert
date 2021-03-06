#version 450
#include "include/camera.glsl"

layout(location=0) in vec2 a_position;

layout(location=0) out vec4 v_position;
layout(location=1) out mat3 v_tbn;
layout(location=4) out vec3 v_normal;
layout(location=5) out float v_size;

layout(set=3, binding=0) uniform Node {
    vec2 u_translation;
    float u_size;
};

layout(set = 6, binding = 0) uniform texture2D t_elvation_normal;
layout(set = 6, binding = 2) uniform sampler t_compute_sampler;

void main() {
    vec2 pos = a_position + u_translation;

    ivec2 sample_pos = ivec2(pos + (u_size / 2));
    vec4 elevation_normal = texelFetch(sampler2D(t_elvation_normal, t_compute_sampler), sample_pos, 0);

    float elev = elevation_normal.x;
    vec3 normal = elevation_normal.yzw;

    vec3 bitangent = normalize(cross(vec3(0.0, 0.0, 1.0), normal));
    vec3 tangent = normalize(cross(normal, bitangent));

    v_position = vec4(a_position.x + u_translation.x, elev, a_position.y + u_translation.y, 1.0);
    v_tbn = mat3(tangent, bitangent, normal);
    v_normal = normal;
    v_size = u_size;

    gl_ClipDistance[0] = dot(v_position, cam.clip);
    gl_Position = cam.view_proj * v_position;
}
