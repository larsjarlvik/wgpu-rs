use super::renderer;
use crate::{
    camera, pipelines,
    world::{bundles, node, WorldData},
};
use cgmath::*;
use pipelines::*;

pub struct Eye {
    pub terrain: bundles::Terrain,
    pub water: bundles::Water,
    pub models: bundles::Models,
    pub models_bundle: wgpu::RenderBundle,
    pub sky: bundles::Sky,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Eye {
    pub fn new(
        device: &wgpu::Device,
        deferred: &deferred::DeferredRender,
        world_data: &WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let deferred = deferred.get_render_bundle(device, &camera, "eye");
        let nodes = root_node.get_nodes(&camera);
        let mut models = bundles::Models::new(device, &world_data.models);
        let models_bundle = models.get_bundle(device, &camera, &world_data, &nodes);

        Self {
            terrain: bundles::Terrain::new(device, &camera, &world_data.terrain, &nodes),
            water: bundles::Water::new(device, &camera, &world_data.water, &nodes),
            models,
            models_bundle,
            sky: bundles::Sky::new(device, &camera, &world_data.sky),
            camera,
            deferred,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_data: &WorldData,
        viewport: &camera::Viewport,
        view: Matrix4<f32>,
        root_node: &node::Node,
    ) {
        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);

        let nodes = root_node.get_nodes(&self.camera);
        self.terrain = bundles::Terrain::new(device, &self.camera, &world_data.terrain, &nodes);
        self.water = bundles::Water::new(device, &self.camera, &world_data.water, &nodes);
        self.models_bundle = self.models.get_bundle(device, &self.camera, &world_data, &nodes);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        deferred_render: &deferred::DeferredRender,
        world_data: &WorldData,
        viewport: &camera::Viewport,
    ) {
        self.camera.resize(viewport.width, viewport.height);
        self.deferred = deferred_render.get_render_bundle(device, &self.camera, "eye");
        self.sky = bundles::Sky::new(device, &self.camera, &world_data.sky);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        deferred: &deferred::DeferredRender,
        world_data: &WorldData,
        target: &wgpu::TextureView,
    ) {
        renderer::render(
            "environment",
            encoder,
            renderer::Args {
                bundles: vec![&self.terrain.render_bundle, &self.models_bundle],
                color_targets: &[&deferred.target.normals_texture_view, &deferred.target.base_color_texture_view],
                depth_target: Some(&deferred.target.depth_texture_view),
                clear_color: true,
                clear_depth: true,
            },
        );
        renderer::render(
            "deferred",
            encoder,
            renderer::Args {
                bundles: vec![&self.deferred],
                color_targets: &[&world_data.sky.texture_view],
                depth_target: Some(&world_data.sky.depth_texture_view),
                clear_color: true,
                clear_depth: true,
            },
        );
        renderer::render(
            "water",
            encoder,
            renderer::Args {
                bundles: vec![&self.water.render_bundle],
                color_targets: &[&world_data.sky.texture_view],
                depth_target: Some(&world_data.sky.depth_texture_view),
                clear_color: false,
                clear_depth: false,
            },
        );
        renderer::render(
            "sky",
            encoder,
            renderer::Args {
                bundles: vec![&self.sky.render_bundle],
                color_targets: &[&target],
                depth_target: None,
                clear_color: true,
                clear_depth: false,
            },
        );
    }
}
