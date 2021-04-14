use super::{node, WorldData};
use crate::{camera, pipelines};
use cgmath::*;
mod eye;
mod reflection;
mod refraction;
mod shadow;

pub struct Views {
    deferred: pipelines::deferred::DeferredRender,
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
            deferred: deferred_render,
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
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());

        self.eye.update(device, queue, world_data, &viewport, view, root_node);
        self.reflection.update(device, queue, world_data, &viewport, root_node);
        self.refraction.update(device, queue, world_data, &viewport, root_node);
        self.deferred.update(queue, viewport, view);
        self.shadow
            .update(device, queue, world_data, &viewport, view, &self.deferred, root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport) {
        self.deferred = pipelines::deferred::DeferredRender::new(&device, &viewport);
        self.eye.resize(device, &self.deferred, world_data, viewport);
        self.reflection.resize(device, &self.deferred, world_data, viewport);
        self.refraction.resize(device, &self.deferred, viewport);
        self.shadow.resize(viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData, target: &wgpu::TextureView) {
        self.shadow.render(encoder, &self.deferred);
        self.reflection.render(encoder, &self.deferred, world_data);
        self.refraction.render(encoder, &self.deferred, world_data);
        self.eye.render(encoder, &self.deferred, world_data, target);
    }
}
