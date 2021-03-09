
vec3 calculate_light(vec3 position, vec3 normal, float shininess, float intensity) {
    vec3 ambient_color = u_light_color * u_ambient_strength;
    vec3 inverse_light_dir = -u_light_dir;

    float diffuse_strength = max(dot(normal, inverse_light_dir), 0.0);
    vec3 diffuse_color = u_light_color * diffuse_strength;

    vec3 view_dir = normalize(u_eye_pos - position);
    vec3 half_dir = normalize(view_dir + inverse_light_dir);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), shininess);
    vec3 specular_color = specular_strength * u_light_color * intensity;

    return ambient_color + (diffuse_color + specular_color) * u_light_intensity;
}
