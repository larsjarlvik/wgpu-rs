#version 450
#define NOISE_SET 2

#define SURFACE_COLOR vec3(0.0078, 0.5176, 0.7)
#define DEPTH_COLOR vec3(0.0039, 0.00196, 0.145)
#define EXTINCTION vec3(7.0, 30.0, 40.0)

layout(location=0) in vec4 v_position;
layout(location=1) in vec3 v_normal;

layout(location=0) out vec4 f_color;

layout(set = 3, binding = 0) uniform texture2D t_depth_texture;
layout(set = 3, binding = 1) uniform texture2D t_refraction;
layout(set = 3, binding = 2) uniform texture2D t_reflection;
layout(set = 3, binding = 3) uniform sampler t_sampler;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    vec3 u_eye_pos;
    float z_near;
    vec3 u_look_at;
    float z_far;
    vec4 u_clip;
    vec2 u_viewport_size;
};

float water_depth(float depth) {
    float floor_distance = z_near * z_far / (z_far + depth * (z_near - z_far));
    float water_distance = z_near * z_far / (z_far + gl_FragCoord.z * (z_near - z_far));
    return floor_distance - water_distance;
}

void main() {
    vec2 fragCoord = vec2(gl_FragCoord.xy / u_viewport_size) + v_normal.xz * 0.01;

    float log_depth = texture(sampler2D(t_depth_texture, t_sampler), fragCoord).r;
    float depth = water_depth(log_depth) * 0.3;

    vec3 ground = texture(sampler2D(t_refraction, t_sampler), fragCoord).rgb;
    vec3 reflection = texture(sampler2D(t_reflection, t_sampler), vec2(1.0 - fragCoord.x, fragCoord.y)).rgb;
    vec3 refraction = mix(mix(ground, SURFACE_COLOR, clamp(depth / 2.0, 0.0, 1.0)), DEPTH_COLOR, clamp(depth / EXTINCTION, 0.0, 1.0));

    vec3 view_dir = normalize(u_eye_pos - v_position.xyz);
    vec3 half_dir = normalize(view_dir + -vec3(0.5, -1.0, 0.0));
    float specular_strength = pow(max(dot(v_normal, half_dir), 0.0), 80.0);
    vec3 specular_color = specular_strength * vec3(1.0, 0.9, 0.5);

    float fresnel = pow(dot(view_dir, vec3(0.0, 1.0, 0.0)), 10.0);
    vec3 water_color = mix(reflection, refraction, clamp(fresnel, 0.25, 1.0));
    water_color = clamp(water_color + specular_color, 0.0, 1.0);
    water_color = mix(ground, water_color, clamp(depth * 4.0, 0.0, 1.0));

    f_color = vec4(water_color, 1.0);
}
