use crate::{camera, models, noise, pipelines, plane, settings};
use cgmath::*;
use std::{time::Instant, usize};
mod assets;
mod bundles;
mod compute;
mod node;
mod views;

pub struct WorldData {
    pub terrain: pipelines::terrain::Terrain,
    pub water: pipelines::water::Water,
    pub model: pipelines::model::Model,
    pub sky: pipelines::sky::Sky,
    pub noise: noise::Noise,
    pub models: models::Models,
    pub heightmap: plane::Plane,
}

pub struct World {
    root_node: node::Node,
    pub data: WorldData,
    pub views: views::Views,
}

impl World {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let water = pipelines::water::Water::new(device, &viewport, &noise);
        let sky = pipelines::sky::Sky::new(device, &viewport);
        let model = pipelines::model::Model::new(device, &viewport);
        let terrain = pipelines::terrain::Terrain::new(device, queue, &viewport, &noise);

        let compute = compute::Compute::new(device, &noise);
        let heightmap = plane::Plane::new(settings::TILE_SIZE * 2u32.pow(settings::TILE_DEPTH));
        let elevation_task = &compute::Task::new("Elevation", &compute.elevation_pipeline, 1, 1);
        let erosion_task = &compute::Task::new("Erosion", &compute.erosion_pipeline, 1, 4);
        let smooth_task = &compute::Task::new("Smooth", &compute.smooth_pipeline, 2, 1);
        let normal_task = &compute::Task::new("Normals", &compute.normal_pipeline, 1, 1);

        let tasks = vec![elevation_task, erosion_task, smooth_task, normal_task];
        let heightmap = compute.compute(device, queue, tasks, &heightmap).await;

        let mut models = models::Models::new();
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, &model, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let mut data = WorldData {
            terrain,
            water,
            noise,
            model,
            sky,
            models,
            heightmap,
        };

        let root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let bundles = views::Views::new(device, &mut data, viewport, &root_node);

        Self {
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
        self.data.water = pipelines::water::Water::new(device, viewport, &self.data.noise);
        self.data.sky = pipelines::sky::Sky::new(device, viewport);
        self.views.resize(device, &self.data, viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.views.render(encoder, &self.data, target);
    }
}

impl WorldData {
    pub fn get_vertex(&self, p: Vector2<f32>) -> &plane::Vertex {
        let half_size = self.heightmap.size as f32 / 2.0;
        let a = self.heightmap.get_index((p.x + half_size) as u32, (p.y + half_size) as u32) as usize;
        let v = self.heightmap.vertices.get(a).unwrap();
        v
    }

    pub fn get_elevation(&self, v: &plane::Vertex, p: Vector2<f32>) -> f32 {
        let d = -(v.position[0] * v.normal[0] + v.position[1] * v.normal[1] + v.position[2] * v.normal[2]);
        -(d + v.normal[2] * p.y + v.normal[0] * p.x) / v.normal[1]
    }
}
