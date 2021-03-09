use crate::{camera, deferred};
use super::{WorldData, node};
mod eye;
mod refraction;
mod reflection;

pub struct Views {
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
}

impl Views {
    pub fn new(device: &wgpu::Device, deferred_render: &deferred::DeferredRender, world_data: &mut WorldData, cc: &camera::Controller, root_node: &node::Node) -> Views {
        Self {
            eye: eye::Eye::new(device, deferred_render, world_data, &cc, root_node),
            refraction: refraction::Refraction::new(device, deferred_render, world_data, &cc, root_node),
            reflection: reflection::Reflection::new(device, deferred_render, world_data, &cc, root_node),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world_data: &mut WorldData, cc: &camera::Controller, root_node: &node::Node) {
        self.eye.update(device, queue, world_data, &cc, root_node);
        self.reflection.update(device, queue, world_data, &cc, root_node);
        self.refraction.update(device, queue, world_data, &cc, root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, cc: &camera::Controller) {
        self.eye.resize(device, world_data, cc);
        self.reflection.resize(device, world_data, cc);
        self.refraction.resize(cc);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData, target: &wgpu::TextureView) {
        self.reflection.render(encoder, deferred_render, world_data);
        self.refraction.render(encoder, deferred_render, world_data);
        self.eye.render(encoder, deferred_render, world_data, target);
    }
}