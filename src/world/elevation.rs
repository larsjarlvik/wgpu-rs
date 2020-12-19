use noise::NoiseFn;

pub fn get(noise: &noise::Fbm, x: f32, z: f32) -> f32 {
    let result = noise.get([(x * 0.01) as f64, (z * 0.01) as f64]);
    return (result * 15.0) as f32;
}
