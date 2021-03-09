use std::time::Instant;

use crate::{camera, deferred, models, noise, settings};
use cgmath::{vec2, Vector2};
mod assets;
mod node;
mod sky;
mod terrain;
mod water;
mod views;

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
    pub views: views::Views,
}

impl World {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        deferred_render: &deferred::DeferredRender,
        cc: &camera::Controller,
    ) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let mut models = models::Models::new(&device, &cc);
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let terrain = terrain::Terrain::new(device, queue, &cc, &noise);
        let water = water::Water::new(device, &cc, &noise);
        let sky = sky::Sky::new(device, &cc);
        let mut data = WorldData {
            terrain,
            water,
            noise,
            models,
            sky,
        };

        let root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let bundles = views::Views::new(device, deferred_render, &mut data, cc, &root_node);

        Self {
            data,
            views: bundles,
            root_node,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cc: &camera::Controller, time: Instant) {
        self.root_node.update(device, queue, &mut self.data, &cc);
        self.data.water.update(queue, time);
        self.views.update(device, queue, &mut self.data, cc, &self.root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, cc: &camera::Controller) {
        self.data.water = water::Water::new(device, cc, &self.data.noise);
        self.data.sky = sky::Sky::new(device, cc);
        self.views.resize(device, &mut self.data, cc);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, target: &wgpu::TextureView) {
        self.views.render(encoder, deferred_render, &self.data, target);
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
