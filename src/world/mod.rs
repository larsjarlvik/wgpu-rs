use crate::{camera, models};
use rand::*;
pub mod terrain;

pub struct World {
    pub terrain: terrain::Terrain,
    rng: rand::prelude::ThreadRng,
}

impl World {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> World {
        models.load_model(&device, &queue, "pine", "pine.glb");
        models.refresh_render_bundle(&device, &camera);

        let rng = rand::thread_rng();
        let terrain = terrain::Terrain::new(&device, &camera);
        World { terrain, rng }
    }

    pub fn update(&mut self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models) {
        self.add_quad(&device, &camera, models, -50.0, -50.0);
        self.add_quad(&device, &camera, models, -50.0, 50.0);
        self.add_quad(&device, &camera, models, 50.0, -50.0);
        self.add_quad(&device, &camera, models, 50.0, 50.0);
        models.write_instance_buffers(&device, "pine");
        models.refresh_render_bundle(&device, &camera);
    }

    fn add_quad(&mut self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models, x: f32, z: f32) {
        self.terrain.add_quad(&device, &camera, x, z);

        for _ in 0..1000 {
            let mx = x + (&self.rng.gen::<f32>() - 0.5) * 100.0;
            let mz = z + (&self.rng.gen::<f32>() - 0.5) * 100.0;
            let my = self.terrain.get_elevation(mx, mz);

            models.add_instance(
                "pine",
                models::data::Instance {
                    transform: {
                        cgmath::Matrix4::from_translation(cgmath::Vector3 { x: mx, y: my, z: mz })
                            * cgmath::Matrix4::from_angle_y(cgmath::Deg(&self.rng.gen::<f32>() * 360.0))
                            * cgmath::Matrix4::from_scale(&self.rng.gen::<f32>() * 2.0 + 0.5)
                    }
                    .into(),
                },
            );
        }
    }
}
