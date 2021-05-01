use std::{time::Instant, usize};

pub use self::task::Task;
use crate::{noise, settings};
use cgmath::*;
mod compute;
mod task;
mod textures;
mod uniforms;

pub struct Map {
    pub compute: compute::Compute,
    pub textures: textures::Textures,
    pub size: u32,
    data: RawData,
}

struct RawData {
    pub elevation_normals: Vec<Vector4<f32>>,
}

impl Map {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, noise: &noise::Noise) -> Self {
        let start = Instant::now();

        let size = settings::TILE_SIZE * 2u32.pow(settings::TILE_DEPTH);
        let textures = textures::Textures::new(device, size);
        let compute = compute::Compute::new(device, noise, &textures, size);

        let elevation_task = &Task::new("Elevation", &compute.elevation_pipeline, 1, 1);
        let erosion_task = &Task::new("Erosion", &compute.erosion_pipeline, 2, 4);
        let smooth_task = &Task::new("Smooth", &compute.smooth_pipeline, 3, 1);
        let normal_task = &Task::new("Normals", &compute.normal_pipeline, 1, 1);

        let tasks = vec![elevation_task, erosion_task, smooth_task, normal_task];
        compute.run(device, queue, tasks);

        let parse = Instant::now();
        let elevation_normals = textures.read_texture(device, queue, &textures.elevation_normal_texture, size).await;
        let data = RawData { elevation_normals };

        println!("Parse data: {} ms", parse.elapsed().as_millis());
        println!("Build map: {} ms", start.elapsed().as_millis());

        Self {
            size,
            textures,
            compute,
            data,
        }
    }

    pub fn get_position_normal(&self, p: Vector2<f32>) -> (Vector3<f32>, Vector3<f32>) {
        let half_size = self.size as f32 / 2.0;
        let (x, z) = ((p.x + half_size) as u32, (p.y + half_size) as u32);
        let a = (z * self.size + x) as usize;

        let v = self.data.elevation_normals[a];
        (vec3(p.x.floor(), v.x, p.y.floor()), vec3(v.y, v.z, v.w))
    }

    pub fn get_smooth_elevation(&self, p: Vector2<f32>, (pos, normal): (Vector3<f32>, Vector3<f32>)) -> f32 {
        let d = -(pos.x * normal.x + pos.y * normal.y + pos.z * normal.z);
        -(d + normal.z * p.y + normal.x * p.x) / normal[1]
    }
}
