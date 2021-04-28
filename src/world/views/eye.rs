use super::renderer;
use crate::{
    camera, pipelines,
    world::{bundles, node, WorldData},
};
use cgmath::*;
use pipelines::*;

pub struct Eye {
    pub terrain_bundle: wgpu::RenderBundle,
    pub water_bundle: wgpu::RenderBundle,
    pub models_bundle: wgpu::RenderBundle,
    pub sky_bundle: wgpu::RenderBundle,
    pub model_instances: bundles::ModelInstances,
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
        let mut model_instances = bundles::ModelInstances::new(device, &world_data.models);

        Self {
            terrain_bundle: bundles::get_terrain_bundle(device, &camera, &world_data, &nodes),
            water_bundle: bundles::get_water_bundle(device, &camera, &world_data, &nodes),
            sky_bundle: bundles::get_sky_bundle(device, &camera, &world_data.sky),
            models_bundle: bundles::get_models_bundle(device, &camera, world_data, &mut model_instances, &nodes),
            model_instances,
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
        self.terrain_bundle = bundles::get_terrain_bundle(device, &self.camera, &world_data, &nodes);
        self.water_bundle = bundles::get_water_bundle(device, &self.camera, &world_data, &nodes);
        self.models_bundle = bundles::get_models_bundle(device, &self.camera, &world_data, &mut self.model_instances, &nodes);
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
        self.sky_bundle = bundles::get_sky_bundle(device, &self.camera, &world_data.sky);
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
                bundles: vec![&self.terrain_bundle, &self.models_bundle],
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
                bundles: vec![&self.water_bundle],
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
                bundles: vec![&self.sky_bundle],
                color_targets: &[&target],
                depth_target: None,
                clear_color: true,
                clear_depth: false,
            },
        );
    }
}
