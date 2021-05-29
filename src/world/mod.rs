use crate::{camera, noise, plane, settings};
use cgmath::*;
use std::{collections::HashMap, time::Instant};
mod enivornment;
mod map;
mod node;
mod node_assets;
mod node_uniforms;
mod systems;
mod views;

pub struct WorldData {
    pub terrain: systems::terrain::Terrain,
    pub water: systems::water::Water,
    pub assets: systems::assets::Assets,
    pub sky: systems::sky::Sky,
    pub noise: noise::Noise,
    pub environment: enivornment::Environment,
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    pub map: map::Map,
}

pub struct World {
    root_node: node::Node,
    pub tile: plane::Plane,
    pub data: WorldData,
    pub views: views::Views,
}

impl World {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Self {
        let tile = plane::Plane::new(settings::TILE_SIZE);
        let lods = (0..=settings::LODS.len())
            .map(|lod| tile.create_indices(&device, lod as u32 + 1))
            .collect();

        let noise = noise::Noise::new(device, queue).await;
        let map = map::Map::new(device, &noise).await;

        let environment = enivornment::Environment::new(device);
        let water = systems::water::Water::new(device, viewport, &noise, &tile, &environment);
        let sky = systems::sky::Sky::new(device, viewport);
        let assets = systems::assets::Assets::new(device, viewport, &noise, &environment);
        let terrain = systems::terrain::Terrain::new(device, queue, viewport, &noise, &tile, &map, &environment);

        let mut data = WorldData {
            terrain,
            water,
            noise,
            assets,
            sky,
            lods,
            map,
            environment,
        };

        let root_node = node::Node::new(0.0, 0.0, 0);
        let views = views::Views::new(device, &mut data, viewport, &root_node);

        Self {
            tile,
            data,
            views,
            root_node,
        }
    }

    pub async fn generate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) {
        self.data.map.generate(device, queue).await;
        self.data.assets.load_assets(device, queue);
        self.root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        self.views = views::Views::new(device, &mut self.data, viewport, &self.root_node);
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport, time: Instant) {
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());

        self.root_node.update(device, &mut self.data, viewport);
        self.data.environment.update(queue, viewport, view, time);
        self.views.update(device, queue, &self.data, viewport, &self.root_node, &view);
    }

    pub fn resize(&mut self, device: &wgpu::Device, viewport: &camera::Viewport) {
        self.data.water = systems::water::Water::new(device, viewport, &self.data.noise, &self.tile, &self.data.environment);
        self.data.sky = systems::sky::Sky::new(device, viewport);
        self.views.resize(device, &self.data, viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, color_target: &wgpu::TextureView, depth_target: &wgpu::TextureView) {
        optick::event!();
        self.views.render(encoder, &self.data, color_target, depth_target);
    }
}
