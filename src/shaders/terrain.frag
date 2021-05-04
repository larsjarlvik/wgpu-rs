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

layout(set = 1, binding = 0) uniform texture2D t_textures[26];
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
#define PEBBLES_GRASS 22
#define PEBBLES_DESERT 24

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

const Biome tundra = Biome(TUNDRA, TUNDRA, TUNDRA, ROCK, MUD);
const Biome snow = Biome(SNOW, SNOW, SNOW, ROCK, SNOW);
const Biome barren = Biome(BARREN, BARREN, BARREN, CLIFF, MUD);
const Biome grassland = Biome(GRASSLAND, FLOWERS, PEBBLES_GRASS, CLIFF, SAND);
const Biome desert = Biome(DESERT, PEBBLES_DESERT, PEBBLES_DESERT, CLIFF, DESERT);
const Biome tropical = Biome(TROPICAL, TROPICAL, TROPICAL, CLIFF, DESERT);

const BiomeRange biomes[BIOME_COUNT] = BiomeRange[BIOME_COUNT](
    BiomeRange(tundra, snow, 0.0, 0.5),
    BiomeRange(barren, tundra, 5.0, 0.3),
    BiomeRange(tundra, grassland, 10.0, 0.3),
    BiomeRange(barren, grassland, 30.0, 0.3),
    BiomeRange(desert, tropical, 50.0, 0.4)
);

Texture get_texture(int index) {
    vec3 coords = v_position.xyz * 0.25;
    Texture t;
    t.base_color = texture(sampler2D(t_textures[index], s_texture), coords.xz).rgb;;
    t.normal = texture(sampler2D(t_textures[index + 1], s_texture), coords.xz).rgb;;
    return t;
}

Texture get_alt_texture(int i1, int i2, float val) {
    if (val > 0.0) return get_texture(i1);
    if (val < 0.0) return get_texture(i2);
    return Texture(vec3(0.0), vec3(0.0));
}

Texture mix_texture(Texture t1, Texture t2, float val) {
    return Texture(mix(t1.base_color, t2.base_color, val), mix(t1.normal, t2.normal, val));
}

Texture get_biome(float temperature, float moisture, float cliff_beach, float alt) {
    Texture t_norm = get_texture(biomes[0].wet.type);
    Texture t_cliff = get_texture(biomes[0].wet.cliff);
    Texture t_decor = get_texture(biomes[0].wet.alt1);

    float min_temp = biomes[0].max_temp;

    for (int i = 1; i < BIOME_COUNT; i ++) {
        BiomeRange b = biomes[i];

        if (temperature >= min_temp - TEMP_OVERLAP && temperature <= b.max_temp) {
            Texture tn;
            Texture tc;
            Texture ta;

            float moist_diff = (moisture - b.moist_split) / MOIST_OVERLAP;
            if (moist_diff <= -1.0) {
                tn = get_texture(b.dry.type);
                tc = get_alt_texture(b.dry.cliff, b.dry.beach, cliff_beach);
                ta = get_alt_texture(b.dry.alt1, b.dry.alt2, alt);
            } else if (moist_diff >= 1.0) {
                tn = get_texture(b.wet.type);
                tc = get_alt_texture(b.wet.cliff, b.wet.beach, cliff_beach);
                ta = get_alt_texture(b.wet.alt1, b.wet.alt2, alt);
            } else {
                tn = mix_texture(get_texture(b.dry.type), get_texture(b.wet.type), moist_diff * 0.5 + 0.5);
                tc = mix_texture(get_alt_texture(b.dry.cliff, b.dry.beach, cliff_beach), get_alt_texture(b.wet.cliff, b.wet.beach, cliff_beach), moist_diff * 0.5 + 0.5);
                ta = mix_texture(get_alt_texture(b.dry.alt1, b.dry.alt2, alt), get_alt_texture(b.wet.alt1, b.wet.alt2, alt), moist_diff * 0.5 + 0.5);
            }

            float temp_diff = (min_temp - temperature) / TEMP_OVERLAP;
            if (temp_diff > 0.0 && temp_diff < 1.0) {
                t_norm = mix_texture(tn, t_norm, temp_diff);
                t_cliff = mix_texture(tc, t_cliff, temp_diff);
                t_decor = mix_texture(ta, t_decor, temp_diff);
            } else {
                t_norm = tn;
                t_cliff = tc;
                t_decor = ta;
            }
        }

        min_temp = b.max_temp;
    }

    return mix_texture(mix_texture(t_norm, t_decor, abs(alt)), t_cliff, abs(cliff_beach));
}

void main() {
    vec4 biome = texture(sampler2D(t_biome, t_compute_sampler), (v_position.xz + v_size / 2) / v_size);
    Texture t = get_biome(biome.x, biome.y, biome.z, biome.w);

    f_normals = vec4(normalize(v_tbn * (t.normal * 2.0 - 1.0)), 1.0);
    f_base_color = vec4(t.base_color, 1.0);
}
