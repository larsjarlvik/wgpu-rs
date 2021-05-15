use super::renderer;
use crate::{
    camera,
    world::{bundles, node, WorldData},
};
use cgmath::*;

pub struct Refraction {
    pub terrain_bundle: wgpu::RenderBundle,
    pub camera: camera::Instance,
}

impl Refraction {
    pub fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, -1.0, 0.0, 1.0]);
        let nodes = root_node.get_nodes(&Box::new(camera.frustum));

        Self {
            terrain_bundle: bundles::get_terrain_bundle(device, &camera, &world_data, &nodes),
            camera,
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
        optick::event!();
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());
        self.camera.update(
            queue,
            viewport.target,
            viewport.eye,
            viewport.proj * view,
            viewport.z_near..viewport.z_far,
        );

        let nodes = root_node.get_nodes(&Box::new(self.camera.frustum));
        self.terrain_bundle = bundles::get_terrain_bundle(device, &self.camera, &world_data, &nodes);
    }

    pub fn resize(&mut self, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData) {
        optick::event!();
        renderer::render(
            "environment",
            encoder,
            renderer::Args {
                bundles: vec![&self.terrain_bundle],
                color_targets: &[&world_data.water.refraction_texture_view],
                depth_target: Some(&world_data.water.refraction_depth_texture_view),
                clear_color: true,
                clear_depth: true,
            },
        );
    }
}
