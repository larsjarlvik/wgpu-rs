use super::renderer;
use crate::{
    camera,
    pipelines::{self, render_targets},
    world::{bundles, node, WorldData},
};
use cgmath::*;

pub struct Refraction {
    pub terrain_bundle: wgpu::RenderBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Refraction {
    pub fn new(
        device: &wgpu::Device,
        deferred: &pipelines::deferred::DeferredRender,
        world_data: &WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, -1.0, 0.0, 1.0]);
        let deferred = deferred.get_render_bundle(device, &camera, "refraction");
        let nodes = root_node.get_nodes(&camera);

        Self {
            terrain_bundle: bundles::get_terrain_bundle(device, &camera, &world_data.terrain, &nodes),
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
        root_node: &node::Node,
    ) {
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());
        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);

        let nodes = root_node.get_nodes(&self.camera);
        self.terrain_bundle = bundles::get_terrain_bundle(device, &self.camera, &world_data.terrain, &nodes);
    }

    pub fn resize(&mut self, device: &wgpu::Device, deferred: &pipelines::deferred::DeferredRender, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
        self.deferred = deferred.get_render_bundle(device, &self.camera, "refraction");
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        deferred: &pipelines::deferred::DeferredRender,
        world_data: &WorldData,
        render_targets: &render_targets::RenderTargets,
    ) {
        renderer::render(
            "environment",
            encoder,
            render_targets,
            renderer::Args {
                bundles: vec![&self.terrain_bundle],
                color_targets: &[&deferred.target.normals_texture_view, &deferred.target.base_color_texture_view],
                depth_target: Some(&deferred.target.depth_texture_view),
                clear_color: true,
                clear_depth: true,
            },
        );
        renderer::render(
            "deferred",
            encoder,
            render_targets,
            renderer::Args {
                bundles: vec![&self.deferred],
                color_targets: &[&world_data.water.refraction_texture_view],
                depth_target: Some(&world_data.water.refraction_depth_texture_view),
                clear_color: true,
                clear_depth: true,
            },
        );
    }
}
