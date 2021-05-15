use super::renderer;
use crate::{
    camera,
    world::{bundles, node, WorldData},
};
use cgmath::*;

pub struct Reflection {
    pub terrain_bundle: wgpu::RenderBundle,
    pub models_bundle: wgpu::RenderBundle,
    pub sky_bundle: wgpu::RenderBundle,
    pub model_instances: bundles::ModelInstances,
    pub camera: camera::Instance,
}

impl Reflection {
    pub fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);

        let nodes = root_node.get_nodes(&camera.frustum);
        let z_far_range = num_traits::Float::sqrt(viewport.z_far.powf(2.0) + viewport.z_far.powf(2.0));
        let nodes = nodes
            .into_iter()
            .filter(|node| node.get_distance(viewport.eye.x, viewport.eye.z) <= z_far_range)
            .collect();
        let mut model_instances = bundles::ModelInstances::new(device, &world_data.models);

        Self {
            terrain_bundle: bundles::get_terrain_bundle(device, &camera, &world_data, &nodes),
            sky_bundle: bundles::get_sky_bundle(device, &camera, &world_data.sky),
            models_bundle: bundles::get_models_bundle(device, &camera, world_data, &mut model_instances, &nodes),
            model_instances,
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

        let nodes = root_node.get_nodes(&self.camera.frustum);
        self.terrain_bundle = bundles::get_terrain_bundle(device, &self.camera, &world_data, &nodes);
        self.models_bundle = bundles::get_models_bundle(device, &self.camera, &world_data, &mut self.model_instances, &nodes);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
        self.sky_bundle = bundles::get_sky_bundle(device, &self.camera, &world_data.sky);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData) {
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
                bundles: vec![&self.terrain_bundle, &self.models_bundle],
                color_targets: &[&world_data.water.reflection_texture_view],
                depth_target: Some(&world_data.water.reflection_depth_texture_view),
                clear_color: false,
                clear_depth: true,
            },
        );
    }
}
