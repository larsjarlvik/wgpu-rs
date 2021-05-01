use std::usize;

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
    data: Option<RawData>,
}

struct RawData {
    pub elevation_normals: Vec<Vector4<f32>>,
}

impl Map {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise) -> Self {
        let size = settings::TILE_SIZE * 2u32.pow(settings::TILE_DEPTH);
        let textures = textures::Textures::new(device, size);
        let compute = compute::Compute::new(device, noise, &textures, size);

        Self {
            size,
            textures,
            compute,
            data: None,
        }
    }

    pub async fn update_data(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let elevation_normals = self
            .textures
            .read_texture(device, queue, &self.textures.elevation_normal_texture, self.size)
            .await;

        self.data = Some(RawData { elevation_normals });
    }

    pub fn get_position_normal(&self, p: Vector2<f32>) -> (Vector3<f32>, Vector3<f32>) {
        match &self.data {
            Some(data) => {
                let half_size = self.size as f32 / 2.0;
                let (x, z) = ((p.x + half_size) as u32, (p.y + half_size) as u32);
                let a = (z * self.size + x) as usize;

                let v = data.elevation_normals[a];
                (vec3(p.x.floor(), v.x, p.y.floor()), vec3(v.y, v.z, v.w))
            }
            None => (vec3(p.x.floor(), 0.0, p.y.floor()), vec3(0.0, 1.0, 0.0)),
        }
    }

    pub fn get_smooth_elevation(&self, p: Vector2<f32>, (pos, normal): (Vector3<f32>, Vector3<f32>)) -> f32 {
        let d = -(pos.x * normal.x + pos.y * normal.y + pos.z * normal.z);
        -(d + normal.z * p.y + normal.x * p.x) / normal[1]
    }
}
