#ifndef NOISE_SET
    #define NOISE_SET 0
#endif

layout(set = NOISE_SET, binding = 0) uniform texture2D t_noise;
layout(set = NOISE_SET, binding = 1) uniform sampler t_noise_sampler;

float random(vec2 st) {
    vec2 size = textureSize(sampler2D(t_noise, t_noise_sampler), 0);
    return texelFetch(sampler2D(t_noise, t_noise_sampler), ivec2(mod(st, size)), 0).r;
}

float noise(vec2 st) {
    vec2 i = floor(st);
    vec2 f = st - i;
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    vec2 u = vec2(f.x * f.x * (3.0 - 2.0 * f.x), f.y * f.y * (3.0 - 2.0 * f.y));
    return mix(a, b, u.x) +
        (c - a) * u.y * (1.0 - u.x) +
        (d - b) * u.x * u.y;
}

float fbm(vec2 st, uint octaves) {
    float v = 0.0;
    float a = 0.5;
    vec2 shift = vec2(100.0);

    mat2 rot = mat2(cos(0.5), sin(0.5), -sin(0.5), cos(0.5));
    for (int i = 0; i < octaves; ++i) {
        v += a * noise(st);
        st = rot * st * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}
