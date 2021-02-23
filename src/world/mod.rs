use std::time::Instant;

use crate::{camera, models, noise, settings};
use cgmath::{vec2, Vector2};
mod assets;
mod bundle_group;
mod node;
mod terrain;
mod water;
mod sky;

pub struct WorldData {
    pub terrain: terrain::Terrain,
    pub water: water::Water,
    pub noise: noise::Noise,
    pub models: models::Models,
    pub sky: sky::Sky,
}

pub struct World {
    root_node: node::Node,
    pub data: WorldData,
    pub eye_bundle: bundle_group::BundleGroup,
    pub refraction_bundle: bundle_group::BundleGroup,
    pub reflection_bundle: bundle_group::BundleGroup,
}

impl World {
    pub async fn new(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        cameras: &camera::Cameras,
    ) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let mut models = models::Models::new(&device, &cameras);
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let terrain = terrain::Terrain::new(device, queue, &cameras, &noise);
        let water = water::Water::new(device, swap_chain_desc, &cameras, &noise);
        let sky = sky::Sky::new(device, swap_chain_desc, &cameras);
        let mut data = WorldData {
            terrain,
            water,
            noise,
            models,
            sky,
        };

        let eye_bundle = bundle_group::BundleGroup::new()
            .with_terrain_bundle(device, &mut data, &cameras.eye_cam, &root_node)
            .with_water_bundle(device, &mut data, &cameras.eye_cam, &root_node)
            .with_models_bundle(device, &mut data, &cameras.eye_cam);

        let refraction_bundle = bundle_group::BundleGroup::new().with_terrain_bundle(device, &mut data, &cameras.eye_cam, &root_node);

        let reflection_bundle = bundle_group::BundleGroup::new()
            .with_terrain_bundle(device, &mut data, &cameras.eye_cam, &root_node)
            .with_models_bundle(device, &mut data, &cameras.eye_cam);

        Self {
            data,
            eye_bundle,
            refraction_bundle,
            reflection_bundle,
            root_node,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cameras: &camera::Cameras, time: Instant) {
        self.root_node.update(device, queue, &mut self.data, &cameras.eye_cam);
        self.data.water.update(queue, time);
        self.eye_bundle.update(device, &mut self.data, &cameras.eye_cam, &self.root_node);
        self.refraction_bundle
            .update(device, &mut self.data, &cameras.refraction_cam, &self.root_node);
        self.reflection_bundle
            .update(device, &mut self.data, &cameras.reflection_cam, &self.root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, cameras: &camera::Cameras) {
        self.data.water = water::Water::new(device, swap_chain_desc, cameras, &self.data.noise);
        self.data.sky = sky::Sky::new(device, swap_chain_desc, cameras);
    }
}

impl WorldData {
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
