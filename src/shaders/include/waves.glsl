
layout(set=1, binding=0) uniform Uniforms {
    vec3 u_light_dir;
    float u_ambient_strength;
    vec3 u_light_color;
    float u_light_intensity;
    float u_time;
};

float get_wave(vec2 xz) {
    vec2 offset1 = vec2(1.0, 0.5) * u_time * 0.000932;
    vec2 offset2 = vec2(0.5, 1.0) * u_time * 0.001334;
    float h1 = noise(xz * 0.213 + offset1);
    float h2 = noise(xz * 3.231 + offset2);
    return -(h1 + h2 + sin(u_time * 0.001) * 0.5) * 0.5;
}
