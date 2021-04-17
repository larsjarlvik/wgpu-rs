use crate::{
    camera, pipelines, settings,
    world::{bundles, node, WorldData},
};
use cgmath::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub struct Cascade {
    pub models: bundles::Models,
    pub models_bundle: wgpu::RenderBundle,
    pub camera: camera::Instance,
    pub i: usize,
}

impl Cascade {
    fn new(device: &wgpu::Device, world_data: &WorldData, viewport: &camera::Viewport, root_node: &node::Node, i: usize) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let nodes = root_node.get_nodes(&camera);
        let mut models = bundles::Models::new(device, &world_data.models);
        let models_bundle = models.get_shadow_bundle(device, &camera, &world_data.model, &world_data.models, &nodes);

        Self {
            models,
            models_bundle,
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
        view: Matrix4<f32>,
        deferred: &pipelines::deferred::DeferredRender,
        root_node: &node::Node,
    ) {
        self.cascades.par_iter_mut().for_each(|c| {
            c.camera.update(queue, viewport.target, viewport.eye, deferred.shadow_matrix[c.i]);
            c.camera.frustum = camera::FrustumCuller::from_matrix(viewport.proj * view);
            let nodes = root_node.get_nodes(&c.camera);
            c.models_bundle = c
                .models
                .get_shadow_bundle(device, &c.camera, &world_data.model, &world_data.models, &nodes);
        });
    }

    pub fn resize(&mut self, viewport: &camera::Viewport) {
        for i in 0..settings::SHADOW_CASCADE_SPLITS.len() {
            self.cascades[i].camera.resize(viewport.width, viewport.height);
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred: &pipelines::deferred::DeferredRender) {
        for i in 0..settings::SHADOW_CASCADE_SPLITS.len() {
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow_render_pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &deferred.target.shadow_texture_view[i],
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                })
                .execute_bundles(std::iter::once(&self.cascades[i].models_bundle));
        }
    }
}
