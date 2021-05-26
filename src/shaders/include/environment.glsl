#include "camera.glsl"

#ifndef ENVIRONMENT_INITIALIZED
    #ifndef ENVIRONMENT_SET
        #define ENVIRONMENT_SET 4
    #endif
    #ifndef ENVIRONMENT_TEXTURE_SET
        #define ENVIRONMENT_TEXTURE_SET 5
    #endif

    #define OFFSET 1.2
    #define CASCADE_COUNT 4

    layout(set=ENVIRONMENT_SET, binding=0) uniform EnvironmentUniforms {
        vec3 light_dir;
        float ambient_strength;
        vec3 light_color;
        float light_intensity;
        mat4 shadow_matrix[CASCADE_COUNT];
        vec4 shadow_split[CASCADE_COUNT];
        float time;
    } env;

    layout(set = ENVIRONMENT_TEXTURE_SET, binding = 0) uniform texture2D t_shadow[3];
    layout(set = ENVIRONMENT_TEXTURE_SET, binding = 1) uniform samplerShadow t_shadow_sampler;

    float get_shadow_factor(vec3 position, int cascade_index) {
        vec4 shadow_coords = env.shadow_matrix[cascade_index] * vec4(position, 1.0);
        if (shadow_coords.w <= 0.0) {
            return 0.0;
        }

        const vec2 flip_correction = vec2(0.5, -0.5);
        vec3 light_local = vec3(
            shadow_coords.xy * flip_correction / shadow_coords.w + 0.5,
            shadow_coords.z / shadow_coords.w
        );

        vec2 texel_size = 1.0 / textureSize(sampler2DShadow(t_shadow[cascade_index], t_shadow_sampler), 0);
        float total = 0.0;

        for (float y = -OFFSET; y <= OFFSET; y += OFFSET) {
            for (float x = -OFFSET; x <= OFFSET; x += OFFSET) {
                vec2 offset = vec2(x, y) * texel_size;
                vec3 uvc = vec3(light_local.xy + offset, light_local.z);
                total += texture(sampler2DShadow(t_shadow[cascade_index], t_shadow_sampler), uvc);
            }
        }

        return total / 9.0;
    }

    float get_shadow(vec3 position) {
        for(int i = 0; i < CASCADE_COUNT; i ++) {
            if (distance(cam.eye_pos, position) < env.shadow_split[i].x) {
                return get_shadow_factor(position, i);
            }
        }
        return 1.0;
    }

    vec3 calculate_light(vec3 position, vec3 normal, float shininess, float intensity, bool apply_shadows) {
        vec3 ambient_color = env.light_color * env.ambient_strength;
        vec3 inverse_light_dir = -env.light_dir;

        float diffuse_strength = max(dot(normal, inverse_light_dir), 0.0);
        vec3 diffuse_color = env.light_color * diffuse_strength;

        vec3 view_dir = normalize(cam.eye_pos - position);
        vec3 half_dir = normalize(view_dir + inverse_light_dir);

        float specular_strength = pow(max(dot(normal, half_dir), 0.0), shininess);
        vec3 specular_color = specular_strength * env.light_color * intensity;

        vec3 light = (diffuse_color + specular_color) * env.light_intensity;
        if (apply_shadows && (light.x > 0.0 || light.y > 0.0 || light.z > 0.0)) {
            light *= get_shadow(position);
        }

        return ambient_color + light;
    }
#endif

#define ENVIRONMENT_INITIALIZED 1