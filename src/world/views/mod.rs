use crate::{camera, deferred};
use super::{WorldData, node};
mod eye;
mod refraction;
mod reflection;

pub struct Views {
    deferred_render: deferred::DeferredRender,
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
}

impl Views {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Views {
        let deferred_render = deferred::DeferredRender::new(&device, &viewport);
        Self {
            eye: eye::Eye::new(device, &deferred_render, world_data, &viewport, root_node),
            refraction: refraction::Refraction::new(device, &deferred_render, world_data, &viewport, root_node),
            reflection: reflection::Reflection::new(device, &deferred_render, world_data, &viewport, root_node),
            deferred_render,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) {
        self.eye.update(device, queue, world_data, &viewport, root_node);
        self.reflection.update(device, queue, world_data, &viewport, root_node);
        self.refraction.update(device, queue, world_data, &viewport, root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport) {
        self.deferred_render = deferred::DeferredRender::new(&device, &viewport);

        self.eye.resize(device, &self.deferred_render, world_data, viewport);
        self.reflection.resize(device, &self.deferred_render, world_data, viewport);
        self.refraction.resize(device, &self.deferred_render, viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world_data: &WorldData, target: &wgpu::TextureView) {
        self.reflection.render(encoder, &self.deferred_render, world_data);
        self.refraction.render(encoder, &self.deferred_render, world_data);
        self.eye.render(encoder, &self.deferred_render, world_data, target);
    }
}