use crate::{camera, deferred};
use super::{WorldData, node};
mod eye;
mod refraction;
mod reflection;

pub struct Bundles {
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
}

impl Bundles {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, cameras: &camera::Cameras, root_node: &node::Node) -> Bundles {
        Self {
            eye: eye::Eye::new(device, world_data, &cameras.eye_cam, root_node),
            refraction: refraction::Refraction::new(device, world_data, &cameras.refraction_cam, root_node),
            reflection: reflection::Reflection::new(device, world_data, &cameras.refraction_cam, root_node),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, cameras: &camera::Cameras, root_node: &node::Node) {
        self.eye.update(device, world_data, &cameras.eye_cam, root_node);
        self.reflection.update(device, world_data, &cameras.reflection_cam, root_node);
        self.refraction.update(device, world_data, &cameras.refraction_cam, root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, cameras: &camera::Cameras) {
        self.eye.resize(device, world_data, &cameras.eye_cam);
        self.reflection.resize(device, world_data, &cameras.reflection_cam);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData, target: &wgpu::TextureView) {
        self.reflection.render(encoder, deferred_render, world_data);
        self.refraction.render(encoder, deferred_render, world_data);
        self.eye.render(encoder, deferred_render, world_data, target);
    }
}