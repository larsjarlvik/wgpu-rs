use noise::NoiseFn;

pub fn get(noise: &noise::OpenSimplex, x: f32, z: f32) -> f32 {
    let mut result = noise.get([(x * 0.01) as f64, (z * 0.01) as f64]) * 45.0;
    result += noise.get([(x * 0.2) as f64, (z * 0.2) as f64]) * 0.5;
    result as f32
}
