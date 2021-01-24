#version 450

const vec3 sky_color = vec3(0.6, 0.8, 0.9);

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_normal;
layout(set = 0, binding = 2) uniform texture2D t_base_color;
layout(set = 0, binding = 3) uniform sampler t_sampler;

layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
};

layout(set=2, binding=0) uniform Camera {
    mat4 u_view;
    mat4 u_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec2 u_viewport_size;
};

float linearize_depth(float d) {
    return z_near * z_far / (z_far + d * (z_near - z_far));
}

vec4 world_pos_from_depth(float depth, vec2 coords, mat4 view_proj) {
    vec4 pos;
    pos.xy = vec2(coords.x, 1.0 - coords.y) * 2.0 - 1.0;
    pos.z = depth;
    pos.w = 1.0;

    pos = inverse(view_proj) * pos;
    pos /= pos.w;
    return pos;
}

vec3 uv2ray(vec2 uv) {
    vec4 camdir = inverse(u_proj) * vec4(uv, 1.0, 1.0);
    camdir = camdir / camdir.w;
    vec3 dir = mat3(inverse(u_view)) * vec3(camdir);
    return normalize(dir);
}

vec3 sky() {

    vec2 uv = 2.0 * vec2(gl_FragCoord.x, gl_FragCoord.y) / u_viewport_size - 1.0;
    vec3 ray_dir = uv2ray(uv);

    vec3 sun = pow(max(dot(ray_dir, -u_light_dir), 0.0), 50.0) * vec3(1, 0.8, 0.3);
    return pow(sun, vec3(2.2));
}

vec3 calculate_light(vec3 position, vec3 normal) {
    vec3 ambient_color = u_light_color * u_ambient_strength;
    vec3 inverse_light_dir = -u_light_dir;

    float diffuse_strength = max(dot(normal, inverse_light_dir), 0.0);
    vec3 diffuse_color = u_light_color * diffuse_strength;

    vec3 view_dir = normalize(u_eye_pos - position);
    vec3 half_dir = normalize(view_dir + inverse_light_dir);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 16);
    vec3 specular_color = specular_strength * u_light_color;

    return ambient_color + (diffuse_color + specular_color) * u_light_intensity;
}

void main() {
    ivec2 fragCoord = ivec2(gl_FragCoord.xy);
    float depth = texelFetch(sampler2D(t_depth_texture, t_sampler), fragCoord, 0).r;

    vec4 position = world_pos_from_depth(depth, gl_FragCoord.xy / u_viewport_size, u_proj * u_view);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);

    vec3 color;
    if (depth < 1.0) {
        float fog = smoothstep(z_far / 4.0, z_far, linearize_depth(depth));
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz));
        // color = mix(color, sky_color, fog);
        color = mix(sky(), color, 0.2);
    } else {
        color = sky();
    }


    f_color = vec4(color, 1.0);
}
