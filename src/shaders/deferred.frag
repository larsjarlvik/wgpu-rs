#version 450

const vec3 sky_color = vec3(0.6, 0.8, 0.9);

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_depth_texture;
layout(set = 0, binding = 1) uniform texture2D t_position;
layout(set = 0, binding = 2) uniform texture2D t_normal;
layout(set = 0, binding = 3) uniform texture2D t_base_color;
layout(set = 1, binding = 0) uniform sampler t_sampler;

layout(set=2, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
};

layout(set=3, binding=0) uniform Camera {
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

vec3 sky(vec3 rayDir) {
    vec3 sun = pow(max(dot(rayDir, normalize(u_light_dir)), 0.0), 12.0) * vec3(1, 0.8, 0.3);
    float theta = atan(rayDir.y / length(vec2(rayDir.x, rayDir.z)));
    float sky_factor = pow(abs(sin(theta)), 0.5);
    vec3 sky = sky_factor * sky_color + (1.0 - sky_factor) * vec3(1.0, 1.0, 0.9);
    return pow(sky + sun, vec3(2.2));
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

    vec4 position = texelFetch(sampler2D(t_position, t_sampler), fragCoord, 0);
    vec4 normal = normalize(texelFetch(sampler2D(t_normal, t_sampler), fragCoord, 0));
    vec4 base_color = texelFetch(sampler2D(t_base_color, t_sampler), fragCoord, 0);


    vec3 cam_front = normalize(u_eye_pos - u_look_at);
    vec3 cam_right = cross(cam_front, vec3(0, 1, 0));
    vec3 cam_up = cross(cam_right, cam_front);
    vec2 uv = (2.0 * gl_FragCoord.xy - u_viewport_size) / u_viewport_size.y;
    vec3 ray_dir = normalize(cam_front + uv.x * cam_right + uv.y * cam_up);

    vec3 color;
    if (depth < 1.0) {
        float fog = smoothstep(u_z_far / 4.0, u_z_far, linearize_depth(depth));
        color = base_color.rgb * calculate_light(position.xyz, normalize(normal.xyz));
        color = mix(color, sky_color, fog);
    } else {
        color = sky(ray_dir);
    }

    f_color = vec4(color, 1.0);
}
