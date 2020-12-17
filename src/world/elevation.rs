pub fn perlin(mut x: f32, mut y: f32, mut z: f32, octave: i32, persistance: f32, mut seed: i32) -> f32 {
    let mut result = 0.0;
    let mut p = 1.0;
    let mut div = 0.0;

    for _i in 0..octave {
        result += p * noise(x, y, z, seed);
        div += p;

        x = 2.0 * x;
        y = 2.0 * y;
        z = 2.0 * z;
        seed = hash(seed);
        p *= persistance;
    }
    return result / div;
}

#[inline(always)]
fn interpol_cos_1d(a: f32, b: f32, x: f32) -> f32 {
    let x_2 = x * x;
    let x_4 = x_2 * x_2;
    return a + (b - a) * (6.0 * x * x_4 - 15.0 * x_4 + 10.0 * x * x_2);
}

fn interpol_cos_2d(a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) -> f32 {
    let x_2 = x * x;
    let x_4 = x_2 * x_2;
    let v = 6.0 * x * x_4 - 15.0 * x_4 + 10.0 * x * x_2;
    let m = a + (b - a) * v;
    let n = c + (d - c) * v;
    return interpol_cos_1d(m, n, y);
}

#[inline(always)]
fn noise(x: f32, y: f32, z: f32, seed: i32) -> f32 {
    let i = x.floor() as i32;
    let j = y.floor() as i32;
    let k = z.floor() as i32;

    let q1 = rand_pos(i, j, k, seed);
    let q2 = rand_pos(i + 1, j, k, seed);
    let q3 = rand_pos(i, j + 1, k, seed);
    let q4 = rand_pos(i + 1, j + 1, k, seed);

    let q = interpol_cos_2d(q1, q2, q3, q4, x - i as f32, y - j as f32);

    let p1 = rand_pos(i, j, k + 1, seed);
    let p2 = rand_pos(i + 1, j, k + 1, seed);
    let p3 = rand_pos(i, j + 1, k + 1, seed);
    let p4 = rand_pos(i + 1, j + 1, k + 1, seed);
    let p = interpol_cos_2d(p1, p2, p3, p4, x - i as f32, y - j as f32);

    return interpol_cos_1d(q, p, z - k as f32);
}

#[inline(always)]
fn rand_pos(x: i32, y: i32, z: i32, seed: i32) -> f32 {
    let a = hash(x + seed);
    let b = hash(y + a);
    let c = hash(z + b);
    let m = 10000000;
    return (((m + (c % m)) % m) as f32) / (m as f32);
}

#[inline(always)]
pub fn hash(b: i32) -> i32 {
    let mut a = b;
    a = a.wrapping_sub(a << 6);
    a ^= a >> 17;
    a = a.wrapping_sub(a << 9);
    a ^= a << 4;
    a = a.wrapping_sub(a << 3);
    a ^= a << 10;
    a ^= a >> 15;
    return a;
}
