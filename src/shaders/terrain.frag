#version 450

#define NOISE_SET 2
#include "include/noise.glsl"

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;
layout(location=4) in vec3 v_normal;
layout(location=5) in vec4 v_biome;

layout(location=0) out vec4 f_normals;
layout(location=1) out vec4 f_base_color;

layout(set = 1, binding = 0) uniform texture2D t_textures[12];
layout(set = 1, binding = 1) uniform sampler s_texture;

#define DESERT 0
#define GRASSLAND 2
#define SNOW 4
#define TROPICAL 6
#define TUNDRA 8
#define BIOME_COUNT 4
#define TEMP_OVERLAP 10.0
#define MOIST_OVERLAP 0.1

struct Texture {
   vec3 base_color;
   vec3 normal;
};

struct Biome {
    int type_dry;
    int type_wet;
    float max_temp;
    float moist_split;
};

Biome biomes[BIOME_COUNT] = Biome[BIOME_COUNT](
    Biome(TUNDRA, SNOW, 0.0, 0.5),
    Biome(TUNDRA, GRASSLAND, 10.0, 0.3),
    Biome(GRASSLAND, GRASSLAND, 30.0, -1.0),
    Biome(DESERT, TROPICAL, 50.0, 0.4)
);

vec3 getTriPlanarTexture(int textureId) {
    vec3 coords = v_position.xyz * 0.25;
    vec3 blending = abs(v_normal);
    blending = normalize(max(blending, 0.00001));
    blending /= vec3(blending.x + blending.y + blending.z);

    vec3 xaxis = texture(sampler2D(t_textures[textureId], s_texture), coords.yz).rgb;
    vec3 yaxis = texture(sampler2D(t_textures[textureId], s_texture), coords.xz).rgb;
    vec3 zaxis = texture(sampler2D(t_textures[textureId], s_texture), coords.xy).rgb;

    return xaxis * blending.x + yaxis * blending.y + zaxis * blending.z;
}

Texture get_texture(int index) {
    Texture t;
    t.base_color = getTriPlanarTexture(index);
    t.normal = getTriPlanarTexture(index + 1);
    return t;
}

Texture get_biome(float temperature, float moisture) {
    Texture t = get_texture(biomes[0].type_wet);
    float min_temp = biomes[0].max_temp;

    for (int i = 1; i < BIOME_COUNT; i ++) {
        Biome b = biomes[i];

        if (temperature >= min_temp - TEMP_OVERLAP && temperature <= b.max_temp) {
            Texture tn;

            float moist_diff = (moisture - b.moist_split) / MOIST_OVERLAP;
            if (moist_diff <= -1.0) {
                tn = get_texture(b.type_dry);
            } else if (moist_diff >= 1.0) {
                tn = get_texture(b.type_wet);
            } else {
                Texture t_dry = get_texture(b.type_dry);
                Texture t_wet = get_texture(b.type_wet);
                tn.base_color = mix(t_dry.base_color, t_wet.base_color, moist_diff * 0.5 + 0.5);
                tn.normal = mix(t_dry.normal, t_wet.normal, moist_diff * 0.5 + 0.5);
            }

            float temp_diff = (min_temp - temperature) / TEMP_OVERLAP;
            if (temp_diff > 0.0 && temp_diff < 1.0) {
                t.base_color = mix(tn.base_color, t.base_color, temp_diff);
                t.normal = mix(tn.normal, t.normal, temp_diff);
            } else {
                t = tn;
            }
        }

        min_temp = b.max_temp;
    }

    return t;
}

void main() {
    Texture t = get_biome(v_biome.x, v_biome.y);

    f_normals = vec4(normalize(v_tbn * (t.normal * 2.0 - 1.0)), 1.0);
    f_base_color = vec4(t.base_color, 1.0);
}
