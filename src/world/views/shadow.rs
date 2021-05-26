use super::renderer;
use crate::{
    camera, settings,
    world::{node, systems, WorldData},
};
use cgmath::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub struct Cascade {
    pub asset_instances: systems::assets::InstanceBufferMap,
    pub models_bundle: wgpu::RenderBundle,
    pub camera: camera::Instance,
    pub i: usize,
}

impl Cascade {
    fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node, i: usize) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let nodes = root_node.get_nodes(&Box::new(camera.frustum));
        let mut asset_instances = world_data.assets.get_instances(device);

        Self {
            models_bundle: world_data
                .assets
                .get_shadow_bundle(device, &camera, world_data, &mut asset_instances, &nodes),
            asset_instances,
            camera,
            i,
        }
    }
}

pub struct Shadow {
    cascades: Vec<Cascade>,
}

impl Shadow {
    pub fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        Self {
            cascades: (0..settings::SHADOW_CASCADE_SPLITS.len())
                .map(|i| Cascade::new(device, world_data, viewport, root_node, i))
                .collect(),
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_data: &WorldData,
        viewport: &camera::Viewport,
        view: &Matrix4<f32>,
        root_node: &node::Node,
    ) {
        optick::event!();
        self.cascades.par_iter_mut().for_each(|c| {
            let nodes = root_node.get_nodes(&Box::new(c.camera.frustum));

            c.camera.update(
                queue,
                viewport.target,
                viewport.eye,
                world_data.environment.shadow_matrix[c.i],
                viewport.z_near..viewport.z_far,
            );
            c.camera.frustum = camera::FrustumCuller::from_matrix(viewport.proj * view);
            c.models_bundle = world_data
                .assets
                .get_shadow_bundle(device, &c.camera, world_data, &mut c.asset_instances, &nodes);
        });
    }

    pub fn resize(&mut self, viewport: &camera::Viewport) {
        for i in 0..settings::SHADOW_CASCADE_SPLITS.len() {
            self.cascades[i].camera.resize(viewport.width, viewport.height);
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData) {
        optick::event!();
        for i in 0..settings::SHADOW_CASCADE_SPLITS.len() {
            renderer::render(
                "shadows",
                encoder,
                renderer::Args {
                    bundles: vec![&self.cascades[i].models_bundle],
                    color_targets: &[],
                    depth_target: Some(&world_data.environment.shadow_texture_view[i]),
                    clear_color: false,
                    clear_depth: true,
                },
            );
        }
    }
}
