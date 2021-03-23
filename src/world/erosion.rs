use crate::plane;
use rand::Rng;

const MAX_ITERATIONS: usize = 100;
const DEPOSITION_RATE: f32 = 0.3;
const EROSION_RATE: f32 = 0.04;
const SPEED: f32 = 0.15;
const FRICTION: f32 = 0.7;
const ITERATION_SCALE: f32 = 0.4;
const RADIUS: f32 = 0.8;

fn set(plane: &mut plane::Plane, index: f32, val: f32) {
    match plane.vertices.get_mut(index as usize) {
        Some(v) => {
            v.position[1] += val;
        }
        None => {}
    }
}

fn change(plane: &mut plane::Plane, x: f32, z: f32, delta: f32) {
    let size = plane.size as f32;

    let xi = x.floor();
    let zi = z.floor();

    let fx = x - xi;
    let fz = z - zi;

    set(plane, xi + zi * size, fx * fz * delta);
    set(plane, xi + 1.0 + zi * size, (1.0 - fx) * fz * delta);
    set(plane, xi + (zi + 1.0) * size, fx * (1.0 - fz) * delta);
    set(plane, xi + 1.0 + (zi + 1.0) * size, (1.0 - fx) * (1.0 - fz) * delta);
}

fn trace(plane: &mut plane::Plane, x: f32, z: f32) {
    let size = plane.size as f32;

    let mut x = x;
    let mut z = z;
    let mut rng = rand::thread_rng();
    let ox = (rng.gen::<f32>() * 2.0 - 1.0) * RADIUS * size;
    let oz = (rng.gen::<f32>() * 2.0 - 1.0) * RADIUS * size;

    let mut sediment = 0.0;
    let mut xp = x;
    let mut zp = z;
    let mut vx = 0.0;
    let mut vz = 0.0;

    for i in 0..MAX_ITERATIONS {
        let index = (z + oz) * (size + 1.0) + (x + ox);

        match plane.vertices.get(index as usize) {
            Some(vert) => {
                let surface_normal = vert.normal;
                if surface_normal[1] == 1.0 {
                    break;
                }

                let deposit = sediment * DEPOSITION_RATE * surface_normal[1];
                let erosion = EROSION_RATE * (1.0 - surface_normal[1]) * 1f32.min(i as f32 * ITERATION_SCALE);

                change(plane, xp, zp, (deposit - erosion) * 50.0);

                vx = FRICTION * vx + surface_normal[0] * SPEED * size;
                vz = FRICTION * vz + surface_normal[2] * SPEED * size;
                xp = x;
                zp = z;
                x += vx;
                z += vz;
                sediment += erosion - deposit;
            }
            None => {
                break;
            }
        }
    }
}

pub fn erode(plane: &mut plane::Plane) {
    let mut rng = rand::thread_rng();

    for _ in 0..100000 {
        let x = rng.gen::<f32>() * plane.size as f32;
        let z = rng.gen::<f32>() * plane.size as f32;
        trace(plane, x, z);
    }
}
