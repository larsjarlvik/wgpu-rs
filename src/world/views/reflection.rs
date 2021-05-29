use super::renderer;
use crate::{
    camera,
    world::{
        node::{self},
        systems, WorldData,
    },
};
use cgmath::*;

pub struct Reflection {
    pub terrain_bundle: wgpu::RenderBundle,
    pub asset_bundle: wgpu::RenderBundle,
    pub sky_bundle: wgpu::RenderBundle,
    pub asset_instances: systems::assets::InstanceBufferMap,
    pub camera: camera::Instance,
}

impl Reflection {
    pub fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);

        let nodes = root_node.get_nodes(&Box::new(camera.frustum));
        let mut asset_instances = world_data.assets.get_instances(device);

        Self {
            terrain_bundle: world_data.terrain.get_bundle(device, &camera, &world_data, &nodes),
            sky_bundle: world_data.sky.get_bundle(device, &camera),
            asset_bundle: world_data
                .assets
                .get_bundle(device, &camera, world_data, &mut asset_instances, &nodes),
            asset_instances,
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
        let view = Matrix4::look_at_rh(
            Point3::new(viewport.eye.x, -viewport.eye.y, viewport.eye.z),
            viewport.target,
            -Vector3::unit_y(),
        );
        self.camera.update(
            queue,
            viewport.target,
            viewport.eye,
            viewport.proj * view,
            viewport.z_near..viewport.z_far,
        );

        let nodes = root_node.get_nodes(&Box::new(self.camera.frustum));
        self.terrain_bundle = world_data.terrain.get_bundle(device, &self.camera, &world_data, &nodes);
        self.asset_bundle = world_data
            .assets
            .get_bundle(device, &self.camera, &world_data, &mut self.asset_instances, &nodes);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
        self.sky_bundle = world_data.sky.get_bundle(device, &self.camera);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData) {
        optick::event!();
        renderer::render(
            "sky",
            encoder,
            renderer::Args {
                bundles: vec![&self.sky_bundle],
                color_targets: &[&world_data.water.reflection_texture_view],
                depth_target: None,
                clear_color: true,
                clear_depth: false,
            },
        );
        renderer::render(
            "environment",
            encoder,
            renderer::Args {
                bundles: vec![&self.terrain_bundle, &self.asset_bundle],
                color_targets: &[&world_data.water.reflection_texture_view],
                depth_target: Some(&world_data.water.reflection_depth_texture_view),
                clear_color: false,
                clear_depth: true,
            },
        );
    }
}
