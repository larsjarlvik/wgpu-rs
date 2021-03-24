use std::time::Instant;

use crate::plane;
use cgmath::*;
use rand::{prelude::ThreadRng, Rng};

const MAX_ITERATIONS: usize = 100;
const DEPOSITION_RATE: f32 = 0.03;
const EROSION_RATE: f32 = 0.04;
const SPEED: f32 = 0.15;
const FRICTION: f32 = 0.7;
const ITERATION_SCALE: f32 = 0.04;
const RADIUS: f32 = 0.8;
const RESOLUTION: f32 = 0.1;
const SCALE: f32 = 4.0;
const ITERATIONS_PER_CELL: f32 = 1.0;

fn set(plane: &mut plane::Plane, index: f32, val: f32) {
    match plane.vertices.get_mut(index as usize) {
        Some(v) => {
            v.position[1] += val;
        }
        None => {}
    }
}

fn get(plane: &plane::Plane, index: f32) -> f32 {
    match plane.vertices.get(index as usize) {
        Some(v) => v.position[1],
        None => 0.0,
    }
}

fn sample(plane: &plane::Plane, x: f32, z: f32) -> f32 {
    let size = (plane.size + 1) as f32;

    let x = x * SCALE;
    let z = z * SCALE;

    let xi = x.floor();
    let zi = z.floor();

    let fx = x - xi;
    let fy = z - zi;
    let ylu = get(plane, xi + zi * size);
    let yld = get(plane, xi + (zi + 1.0) * size);
    let yru = get(plane, xi + 1.0 + zi * size);
    let yrd = get(plane, xi + 1.0 + (zi + 1.0) * size);
    let yl = ylu + (yld - ylu) * fy;
    let yr = yru + (yrd - yru) * fy;

    return yl + (yr - yl) * fx;
}

fn sample_normal(plane: &plane::Plane, x: f32, z: f32) -> Vector3<f32> {
    let double_radius = -(RESOLUTION + RESOLUTION);
    let left = sample(plane, x - RESOLUTION, z);
    let top = sample(plane, x, z - RESOLUTION);
    let right = sample(plane, x + RESOLUTION, z);
    let bottom = sample(plane, x, z + RESOLUTION);

    let normal = Vector3::new(
        double_radius * (right - left),
        double_radius * double_radius,
        double_radius * (bottom - top),
    )
    .normalize();
    normal
}

fn change(plane: &mut plane::Plane, x: f32, z: f32, delta: f32) {
    let size = (plane.size + 1) as f32;

    let x = x * SCALE;
    let z = z * SCALE;

    let xi = x.floor();
    let zi = z.floor();

    let fx = x - xi;
    let fz = z - zi;

    set(plane, xi + zi * size, fx * fz * delta);
    set(plane, xi + 1.0 + zi * size, (1.0 - fx) * fz * delta);
    set(plane, xi + (zi + 1.0) * size, fx * (1.0 - fz) * delta);
    set(plane, xi + 1.0 + (zi + 1.0) * size, (1.0 - fx) * (1.0 - fz) * delta);
}

fn trace(plane: &mut plane::Plane, x: f32, z: f32, rng: &mut ThreadRng) {
    let mut x = x;
    let mut z = z;
    let ox = (rng.gen::<f32>() * 2.0 - 1.0) * RADIUS * RESOLUTION;
    let oz = (rng.gen::<f32>() * 2.0 - 1.0) * RADIUS * RESOLUTION;

    let mut sediment = 0.0;
    let mut xp = x;
    let mut zp = z;
    let mut vx = 0.0;
    let mut vz = 0.0;

    for i in 0..MAX_ITERATIONS {
        let surface_normal = sample_normal(&plane, x + ox, z + oz);
        if surface_normal.y == 1.0 {
            break;
        }

        let deposit = sediment * DEPOSITION_RATE * surface_normal.y;
        let erosion = EROSION_RATE * (1.0 - surface_normal.y) * 1f32.min(i as f32 * ITERATION_SCALE);

        change(plane, xp, zp, deposit - erosion);

        vx = FRICTION * vx + surface_normal.x * SPEED * RESOLUTION;
        vz = FRICTION * vz + surface_normal.z * SPEED * RESOLUTION;
        xp = x;
        zp = z;
        x += vx;
        z += vz;

        sediment += erosion - deposit;
    }
}

pub fn erode(plane: &mut plane::Plane) {
    let now = Instant::now();
    let mut rng = rand::thread_rng();
    let balls = (ITERATIONS_PER_CELL * plane.length as f32) as usize;

    for _ in 0..balls {
        let x = rng.gen::<f32>() * plane.size as f32;
        let z = rng.gen::<f32>() * plane.size as f32;
        trace(plane, x, z, &mut rng);
    }

    println!("Erosion: {} ms", now.elapsed().as_millis());
}
