#define NUM_OCTAVES 6

layout(set = 2, binding = 0) uniform texture2D t_noise;
layout(set = 2, binding = 1) uniform sampler t_noise_sampler;

float random(vec2 st) {
    vec2 size = textureSize(sampler2D(t_noise, t_noise_sampler), 0);
    return texelFetch(sampler2D(t_noise, t_noise_sampler), ivec2(mod(st, size)), 0).r;
}

float noise(vec2 _st) {
    vec2 i = floor(_st);
    vec2 f = fract(_st);
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    vec2 u = f * f * vec2(3.0 - 2.0 * f.x, 3.0 - 2.0 * f.y);
    return mix(a, b, u.x) +
        (c - a) * u.y * (1.0 - u.x) +
        (d - b) * u.x * u.y;
}

float fbm(vec2 st) {
    float v = 0.0;
    float a = 0.5;
    vec2 shift = vec2(100.0);

    for (int i = 0; i < 6; ++i) {
        v += a * noise(st);
        st = st * 2.0 + shift;
        a *= 0.5;
    }
    return v;
}
