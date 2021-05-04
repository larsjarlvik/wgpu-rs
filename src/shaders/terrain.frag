#version 450

#define NOISE_SET 2
#include "include/noise.glsl"


layout(set = 4, binding = 1) uniform texture2D t_biome;
layout(set = 4, binding = 2) uniform sampler t_compute_sampler;

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;
layout(location=4) in vec3 v_normal;
layout(location=5) in float v_size;

layout(location=0) out vec4 f_normals;
layout(location=1) out vec4 f_base_color;

layout(set = 1, binding = 0) uniform texture2D t_textures[22];
layout(set = 1, binding = 1) uniform sampler s_texture;

#define DESERT 0
#define GRASSLAND 2
#define SNOW 4
#define TROPICAL 6
#define TUNDRA 8
#define BARREN 10
#define ROCK 12
#define CLIFF 14
#define SAND 16
#define FLOWERS 18
#define MUD 20

#define BIOME_COUNT 5
#define TEMP_OVERLAP 10.0
#define MOIST_OVERLAP 0.1

struct Texture {
   vec3 base_color;
   vec3 normal;
};

struct Biome {
    int type;
    int alt1;
    int alt2;
    int cliff;
    int beach;
};

struct BiomeRange {
    Biome dry;
    Biome wet;
    float max_temp;
    float moist_split;
};

const Biome tundra = Biome(TUNDRA, MUD, ROCK, ROCK, MUD);
const Biome snow = Biome(SNOW, TUNDRA, ROCK, ROCK, SNOW);
const Biome barren = Biome(BARREN, MUD, ROCK, CLIFF, MUD);
const Biome grassland = Biome(GRASSLAND, FLOWERS, MUD, CLIFF, SAND);
const Biome desert = Biome(DESERT, SAND, SAND, CLIFF, DESERT);
const Biome tropical = Biome(TROPICAL, GRASSLAND, MUD, CLIFF, DESERT);

const BiomeRange biomes[BIOME_COUNT] = BiomeRange[BIOME_COUNT](
    BiomeRange(tundra, snow, 0.0, 0.5),
    BiomeRange(barren, tundra, 5.0, 0.3),
    BiomeRange(tundra, grassland, 10.0, 0.3),
    BiomeRange(barren, grassland, 30.0, 0.3),
    BiomeRange(desert, tropical, 50.0, 0.4)
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

Texture get_alt_texture(int i1, int i2, float val) {
    if (val > 0.0) return get_texture(i1);
    if (val < 0.0) return get_texture(i2);
    return Texture(vec3(0.0), vec3(0.0));
}


Texture get_biome(float temperature, float moisture, float cliff_beach) {
    Texture t_norm = get_texture(biomes[0].wet.type);
    Texture t_cliff = get_texture(biomes[0].wet.cliff);

    float min_temp = biomes[0].max_temp;

    for (int i = 1; i < BIOME_COUNT; i ++) {
        BiomeRange b = biomes[i];

        if (temperature >= min_temp - TEMP_OVERLAP && temperature <= b.max_temp) {
            Texture tn;
            Texture tc;

            float moist_diff = (moisture - b.moist_split) / MOIST_OVERLAP;
            if (moist_diff <= -1.0) {
                tn = get_texture(b.dry.type);
                tc = get_alt_texture(b.dry.cliff, b.dry.beach, cliff_beach);
            } else if (moist_diff >= 1.0) {
                tn = get_texture(b.wet.type);
                tc = get_alt_texture(b.wet.cliff, b.wet.beach, cliff_beach);
            } else {
                Texture tn_dry = get_texture(b.dry.type);
                Texture tn_wet = get_texture(b.wet.type);
                tn.base_color = mix(tn_dry.base_color, tn_wet.base_color, moist_diff * 0.5 + 0.5);
                tn.normal = mix(tn_dry.normal, tn_wet.normal, moist_diff * 0.5 + 0.5);

                Texture tc_dry = get_alt_texture(b.dry.cliff, b.dry.beach, cliff_beach);
                Texture tc_wet = get_alt_texture(b.wet.cliff, b.wet.beach, cliff_beach);
                tc.base_color = mix(tc_dry.base_color, tc_wet.base_color, moist_diff * 0.5 + 0.5);
                tc.normal = mix(tc_dry.normal, tc_wet.normal, moist_diff * 0.5 + 0.5);
            }

            float temp_diff = (min_temp - temperature) / TEMP_OVERLAP;
            if (temp_diff > 0.0 && temp_diff < 1.0) {
                t_norm.base_color = mix(tn.base_color, t_norm.base_color, temp_diff);
                t_norm.normal = mix(tn.normal, t_norm.normal, temp_diff);

                t_cliff.base_color = mix(tc.base_color, t_cliff.base_color, temp_diff);
                t_cliff.normal = mix(tc.normal, t_cliff.normal, temp_diff);
            } else {
                t_norm = tn;
                t_cliff = tc;
            }
        }

        min_temp = b.max_temp;
    }

    Texture result;
    result.base_color = mix(t_norm.base_color, t_cliff.base_color, abs(cliff_beach));
    result.normal = mix(t_norm.normal, t_cliff.normal, abs(cliff_beach));
    return result;
}

void main() {
    vec4 biome = texture(sampler2D(t_biome, t_compute_sampler), (v_position.xz + v_size / 2) / v_size);
    Texture t = get_biome(biome.x, biome.y, biome.z);

    f_normals = vec4(normalize(v_tbn * (t.normal * 2.0 - 1.0)), 1.0);
    f_base_color = vec4(t.base_color, 1.0);
}
