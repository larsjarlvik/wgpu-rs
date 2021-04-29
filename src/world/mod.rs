use crate::{assets, camera, models, noise, pipelines, plane, settings};
use std::{collections::HashMap, time::Instant};
mod bundles;
mod compute;
mod node;
mod node_uniforms;
mod views;

pub struct WorldData {
    pub terrain: pipelines::terrain::Terrain,
    pub water: pipelines::water::Water,
    pub model: pipelines::model::Model,
    pub sky: pipelines::sky::Sky,
    pub noise: noise::Noise,
    pub models: models::Models,
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    pub map: compute::ComputeBindings,
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

        let noise = noise::Noise::new(&device, &queue).await;

        let compute = compute::Compute::new(device, &noise);
        let elevation_task = &compute::Task::new("Elevation", &compute.elevation_pipeline, 1, 1);
        let erosion_task = &compute::Task::new("Erosion", &compute.erosion_pipeline, 1, 4);
        let smooth_task = &compute::Task::new("Smooth", &compute.smooth_pipeline, 2, 1);
        let normal_task = &compute::Task::new("Normals", &compute.normal_pipeline, 1, 1);

        let tasks = vec![elevation_task, erosion_task, smooth_task, normal_task];
        compute.compute(device, queue, tasks).await;

        let map = compute.create_bindings(device);

        let water = pipelines::water::Water::new(device, &viewport, &noise, &tile);
        let sky = pipelines::sky::Sky::new(device, &viewport);
        let model = pipelines::model::Model::new(device, &viewport);
        let terrain = pipelines::terrain::Terrain::new(device, queue, &viewport, &noise, &tile, &map.bind_group_layout);

        let now = Instant::now();
        let mut models = models::Models::new();
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, &model, &asset);
        }
        println!("Assets: {} ms", now.elapsed().as_millis());

        let mut data = WorldData {
            terrain,
            water,
            noise,
            model,
            sky,
            models,
            lods,
            map,
        };

        let root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let bundles = views::Views::new(device, &mut data, viewport, &root_node);

        Self {
            tile,
            data,
            views: bundles,
            root_node,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport, time: Instant) {
        self.root_node.update(device, &mut self.data, viewport);
        self.data.water.update(queue, time);
        self.views.update(device, queue, &self.data, viewport, &self.root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, viewport: &camera::Viewport) {
        self.data.water = pipelines::water::Water::new(device, viewport, &self.data.noise, &self.tile);
        self.data.sky = pipelines::sky::Sky::new(device, viewport);
        self.views.resize(device, &self.data, viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.views.render(encoder, &self.data, target);
    }
}
