#version 450
#extension GL_GOOGLE_include_directive : require
#include "include/noise.glsl"

layout(location=0) in vec4 v_position;
layout(location=1) in mat3 v_tbn;
layout(location=4) in vec3 v_normal;

layout(location=0) out vec4 f_position;
layout(location=1) out vec4 f_normal;
layout(location=2) out vec4 f_base_color;

layout(set = 1, binding = 0) uniform texture2D t_textures[5];
layout(set = 1, binding = 1) uniform sampler s_texture;

struct Texture {
   vec3 diffuse;
   vec3 normal;
};


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

Texture get_texture() {
    vec3 grass = getTriPlanarTexture(0);
    vec3 sand = getTriPlanarTexture(4);
    vec3 cliffs = getTriPlanarTexture(2);

    // TODO: Improve look and performance
    vec3 ground = mix(sand, grass, clamp(v_position.y + (fbm(v_position.xz * 2.5, 2)) * 4.0, 0.0, 1.0));

    vec3 grass_sand_normal = getTriPlanarTexture(1);
    vec3 cliff_normal = getTriPlanarTexture(3);

    float theta = clamp(0.0, 1.0, pow(1.0 - max(dot(v_normal, vec3(0, 1, 0)), 0.0), 1.2) * 6.0);
    Texture t;
    t.diffuse = mix(ground, cliffs, theta);
    t.normal = mix(grass_sand_normal, cliff_normal, theta);
    return t;
}

void main() {
    Texture t = get_texture();

    f_position = v_position;
    f_normal = vec4(normalize(v_tbn * (t.normal * 2.0 - 1.0)), 1.0);
    f_base_color = vec4(t.diffuse, 1.0);
}
