use crate::{camera, models};
use rand::*;
use rayon::prelude::*;
use std::time::Instant;
mod elevation;
pub mod terrain;

pub struct World {
    pub terrain: terrain::Terrain,
    pub tile_size: u32,
}

impl World {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> World {
        models.load_model(&device, &queue, "pine", "pine.glb");
        models.refresh_render_bundle(&device, &camera);
        let terrain = terrain::Terrain::new(&device, &camera);
        World { terrain, tile_size: 20 }
    }

    pub fn update(&mut self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models) {
        let now = Instant::now();
        let range = 100.0;
        let x_l = (camera.target.x - range) as i32;
        let x_u = (camera.target.x + range) as i32;
        let z_l = (camera.target.z - range) as i32;
        let z_u = (camera.target.z + range) as i32;

        for z in z_l..z_u {
            for x in x_l..x_u {
                if x % self.tile_size as i32 == 0 && z % self.tile_size as i32 == 0 {
                    self.add_tile(&device, models, x as f32, z as f32);
                }
            }
        }

        self.terrain.build(&device, &camera);
        models.write_instance_buffers(&device, "pine");
        models.refresh_render_bundle(&device, &camera);

        println!("Generate tiles: {} ms", now.elapsed().as_millis());
    }

    fn add_tile(&mut self, device: &wgpu::Device, models: &mut models::Models, x: f32, z: f32) {
        self.terrain.add_tile(&device, x, z, self.tile_size as f32);

        let density = 0.5;
        let count = (self.tile_size as f32 * density) as u32;

        let terrain = &self.terrain;
        let tile_size = self.tile_size as f32;
        let instances = (0..count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mx = x + (&rng.gen::<f32>() - 0.5) * tile_size;
                let mz = z + (&rng.gen::<f32>() - 0.5) * tile_size;
                let my = terrain.get_elevation(mx, mz);
                models::data::Instance {
                    transform: {
                        cgmath::Matrix4::from_translation(cgmath::Vector3 { x: mx, y: my, z: mz })
                            * cgmath::Matrix4::from_angle_y(cgmath::Deg(&rng.gen::<f32>() * 360.0))
                            * cgmath::Matrix4::from_scale(&rng.gen::<f32>() * 2.0 + 0.5)
                    }
                    .into(),
                }
            })
            .collect::<Vec<models::data::Instance>>();

        for instance in instances {
            models.add_instance("pine", instance);
        }
    }
}
