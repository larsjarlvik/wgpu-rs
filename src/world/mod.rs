use crate::{camera, noise, pipelines, plane, settings};
use cgmath::*;
use std::{collections::HashMap, time::Instant};
mod bundles;
mod lights;
mod map;
mod node;
mod node_assets;
mod node_uniforms;
mod views;

pub struct WorldData {
    pub terrain: pipelines::terrain::Terrain,
    pub water: pipelines::water::Water,
    pub model: pipelines::assets::Assets,
    pub sky: pipelines::sky::Sky,
    pub noise: noise::Noise,
    pub lights: lights::Lights,
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
        let map = map::Map::new(device, queue, &noise).await;

        let lights = lights::Lights::new(device);
        let water = pipelines::water::Water::new(
            device,
            viewport,
            &noise,
            &tile,
            &lights.uniform_bind_group_layout,
            &lights.texture_bind_group_layout,
        );
        let sky = pipelines::sky::Sky::new(device, viewport);
        let model = pipelines::assets::Assets::new(
            device,
            queue,
            viewport,
            &lights.uniform_bind_group_layout,
            &lights.texture_bind_group_layout,
        );
        let terrain = pipelines::terrain::Terrain::new(
            device,
            queue,
            viewport,
            &noise,
            &tile,
            &map.textures.bind_group_layout,
            &lights.uniform_bind_group_layout,
            &lights.texture_bind_group_layout,
        );

        let mut data = WorldData {
            terrain,
            water,
            noise,
            model,
            sky,
            lods,
            map,
            lights,
        };

        let root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let views = views::Views::new(device, &mut data, viewport, &root_node);

        Self {
            tile,
            data,
            views,
            root_node,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport, time: Instant) {
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());

        self.root_node.update(device, &mut self.data, viewport);

        self.data.lights.update(queue, viewport, view);
        self.data.water.update(queue, time);
        self.views.update(device, queue, &self.data, viewport, &self.root_node, &view);
    }

    pub fn resize(&mut self, device: &wgpu::Device, viewport: &camera::Viewport) {
        self.data.water = pipelines::water::Water::new(
            device,
            viewport,
            &self.data.noise,
            &self.tile,
            &self.data.lights.uniform_bind_group_layout,
            &self.data.lights.texture_bind_group_layout,
        );
        self.data.sky = pipelines::sky::Sky::new(device, viewport);
        self.views.resize(device, &self.data, viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, color_target: &wgpu::TextureView, depth_target: &wgpu::TextureView) {
        optick::event!();
        self.views.render(encoder, &self.data, color_target, depth_target);
    }
}
