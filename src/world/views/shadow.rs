use crate::{
    camera, pipelines,
    world::{bundles, node, WorldData},
};

pub struct Shadow {
    pub models: bundles::Models,
    pub camera: camera::Instance,
}

impl Shadow {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let nodes = root_node.get_nodes(&camera);

        Self {
            models: bundles::Models::new_shadow_bundle(device, &camera, &world_data.model, &mut world_data.models, &nodes),
            camera,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        deferred: &pipelines::deferred::DeferredRender,
        root_node: &node::Node,
    ) {
        self.camera.update(queue, viewport.target, viewport.eye, deferred.shadow_matrix);

        let nodes = root_node.get_nodes(&self.camera);
        self.models = bundles::Models::new_shadow_bundle(device, &self.camera, &world_data.model, &mut world_data.models, &nodes);
    }

    pub fn resize(&mut self, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred: &pipelines::deferred::DeferredRender) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow_render_pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &deferred.target.shadow_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(std::iter::once(&self.models.render_bundle));
    }
}
