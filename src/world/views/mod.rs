use super::{node, WorldData};
use crate::{camera, pipelines};
mod eye;
mod reflection;
mod refraction;
mod shadow;

pub struct Views {
    deferred_render: pipelines::deferred::DeferredRender,
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
    shadow: shadow::Shadow,
}

impl Views {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Views {
        let deferred_render = pipelines::deferred::DeferredRender::new(&device, &viewport);
        Self {
            eye: eye::Eye::new(device, &deferred_render, world_data, &viewport, root_node),
            refraction: refraction::Refraction::new(device, &deferred_render, world_data, &viewport, root_node),
            reflection: reflection::Reflection::new(device, &deferred_render, world_data, &viewport, root_node),
            shadow: shadow::Shadow::new(device, world_data, &viewport, root_node),
            deferred_render,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) {
        self.deferred_render.update(queue, &self.eye.camera);
        self.eye.update(device, queue, world_data, &viewport, root_node);
        self.reflection.update(device, queue, world_data, &viewport, root_node);
        self.refraction.update(device, queue, world_data, &viewport, root_node);
        self.shadow
            .update(device, queue, world_data, &viewport, &self.deferred_render, root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport) {
        self.deferred_render = pipelines::deferred::DeferredRender::new(&device, &viewport);

        self.eye.resize(device, &self.deferred_render, world_data, viewport);
        self.reflection.resize(device, &self.deferred_render, world_data, viewport);
        self.refraction.resize(device, &self.deferred_render, viewport);
        self.shadow.resize(viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData, target: &wgpu::TextureView) {
        self.shadow.render(encoder, &self.deferred_render);
        self.reflection.render(encoder, &self.deferred_render, world_data);
        self.refraction.render(encoder, &self.deferred_render, world_data);
        self.eye.render(encoder, &self.deferred_render, world_data, target);
    }
}
