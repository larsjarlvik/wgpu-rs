use crate::{camera, models, noise, settings};
use cgmath::{vec2, Vector2};
use std::time::Instant;
mod assets;
mod node;
mod terrain;

pub struct World {
    root_node: Option<node::Node>,
    terrain: terrain::Terrain,
    noise: noise::Noise,
    pub models: models::Models,
    pub terrain_bundle: wgpu::RenderBundle,
}

impl World {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let mut models = models::Models::new(&device, &camera);
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let now = Instant::now();
        let terrain = terrain::Terrain::new(device, queue, camera, &noise);
        let terrain_bundle = get_terrain_bundle(device, camera, &terrain, &None);

        let mut world = Self {
            terrain,
            terrain_bundle,
            root_node: None,
            noise,
            models,
        };

        world.root_node = Some(node::Node::new(0.0, 0.0, settings::TILE_DEPTH, device, queue, &mut world));
        device.poll(wgpu::Maintain::Wait);

        world.terrain_bundle = get_terrain_bundle(device, camera, &world.terrain, &world.root_node);
        println!("Generate world: {} ms", now.elapsed().as_millis());
        world
    }

    pub fn refresh(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.models.refresh_render_bundle(device, camera);
        self.terrain_bundle = get_terrain_bundle(device, camera, &self.terrain, &self.root_node);
    }

    pub fn get_elevation(&self, p: Vector2<f32>) -> f32 {
        let xz = p * settings::TERRAIN_SCALE;
        let q = vec2(
            self.noise.fbm(xz, settings::TERRAIN_OCTAVES),
            self.noise.fbm(xz + vec2(1.0, 1.0), settings::TERRAIN_OCTAVES),
        );

        let r = vec2(
            self.noise.fbm(xz + q + vec2(1.7 + 0.15, 9.2 + 0.15), settings::TERRAIN_OCTAVES),
            self.noise.fbm(xz + q + vec2(8.3 + 0.126, 2.8 + 0.126), settings::TERRAIN_OCTAVES),
        );

        (self.noise.fbm(xz + r, settings::TERRAIN_OCTAVES) - 0.3) / settings::TERRAIN_SCALE / 2.0
    }
}

fn get_terrain_bundle(
    device: &wgpu::Device,
    camera: &camera::Camera,
    terrain: &terrain::Terrain,
    root_node: &Option<node::Node>,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
        ],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&terrain.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &terrain.texture_bind_group, &[]);
    encoder.set_bind_group(2, &terrain.noise_bindings.bind_group, &[]);

    match root_node {
        Some(rn) => {
            for slice in rn.get_terrain_buffer_slices(camera) {
                encoder.set_vertex_buffer(0, slice);
                encoder.draw(0..terrain.compute.num_elements, 0..1);
            }
        }
        None => {}
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}
