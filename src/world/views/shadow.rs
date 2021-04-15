use crate::{
    camera, pipelines, settings,
    world::{bundles, node, WorldData},
};
use cgmath::*;

pub struct Cascade {
    pub models: bundles::Models,
    pub camera: camera::Instance,
}

impl Cascade {
    fn new(device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let nodes = root_node.get_nodes(&camera);

        Self {
            models: bundles::Models::new_shadow_bundle(device, &camera, &world_data.model, &mut world_data.models, &nodes),
            camera,
        }
    }
}

pub struct Shadow {
    cascades: Vec<Cascade>,
}

impl Shadow {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        Self {
            cascades: vec![
                Cascade::new(device, world_data, viewport, root_node),
                Cascade::new(device, world_data, viewport, root_node),
                Cascade::new(device, world_data, viewport, root_node),
            ],
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        view: Matrix4<f32>,
        deferred: &pipelines::deferred::DeferredRender,
        root_node: &node::Node,
    ) {
        for i in 0..settings::SHADOW_CASCADE_COUNT {
            self.cascades[i]
                .camera
                .update(queue, viewport.target, viewport.eye, deferred.shadow_matrix[i]);
            self.cascades[i].camera.frustum = camera::FrustumCuller::from_matrix(viewport.proj * view);

            let nodes = root_node.get_nodes(&self.cascades[i].camera);
            self.cascades[i].models =
                bundles::Models::new_shadow_bundle(device, &self.cascades[i].camera, &world_data.model, &mut world_data.models, &nodes);
        }
    }

    pub fn resize(&mut self, viewport: &camera::Viewport) {
        for i in 0..settings::SHADOW_CASCADE_COUNT {
            self.cascades[i].camera.resize(viewport.width, viewport.height);
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred: &pipelines::deferred::DeferredRender) {
        for i in 0..settings::SHADOW_CASCADE_COUNT {
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
                .execute_bundles(std::iter::once(&self.cascades[i].models.render_bundle));
        }
    }
}
