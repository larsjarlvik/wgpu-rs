use super::{node, WorldData};
use crate::camera;
use cgmath::*;
mod eye;
mod reflection;
mod refraction;
mod renderer;
mod shadow;

pub struct Views {
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
    shadow: shadow::Shadow,
}

impl Views {
    pub fn new(device: &wgpu::Device, world: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Views {
        Self {
            eye: eye::Eye::new(device, world, viewport, root_node),
            refraction: refraction::Refraction::new(device, world, viewport, root_node),
            reflection: reflection::Reflection::new(device, world, viewport, root_node),
            shadow: shadow::Shadow::new(device, world, viewport, root_node),
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
        view: &Matrix4<f32>,
    ) {
        crossbeam_utils::thread::scope(|scope| {
            let eye = &mut self.eye;
            let reflection = &mut self.reflection;
            let refraction = &mut self.refraction;
            let shadow = &mut self.shadow;

            scope.spawn(move |_| {
                eye.update(device, queue, world, viewport, view, root_node);
            });
            scope.spawn(move |_| {
                reflection.update(device, queue, world, viewport, root_node);
            });
            scope.spawn(move |_| {
                refraction.update(device, queue, world, viewport, root_node);
            });
            scope.spawn(move |_| {
                shadow.update(device, queue, world, viewport, view, root_node);
            });
        })
        .unwrap();
    }

    pub fn resize(&mut self, device: &wgpu::Device, world: &WorldData, viewport: &camera::Viewport) {
        self.eye.resize(device, world, viewport);
        self.reflection.resize(device, world, viewport);
        self.refraction.resize(viewport);
        self.shadow.resize(viewport);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        world: &WorldData,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
    ) {
        self.shadow.render(encoder, world);
        self.reflection.render(encoder, world);
        self.refraction.render(encoder, world);
        self.eye.render(encoder, color_target, depth_target);
    }
}
