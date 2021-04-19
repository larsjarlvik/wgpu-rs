use super::{node, WorldData};
use crate::{camera, pipelines};
use cgmath::*;
mod eye;
mod reflection;
mod refraction;
mod renderer;
mod shadow;

pub struct Views {
    deferred: pipelines::deferred::DeferredRender,
    eye: eye::Eye,
    refraction: refraction::Refraction,
    reflection: reflection::Reflection,
    shadow: shadow::Shadow,
}

impl Views {
    pub fn new(device: &wgpu::Device, world: &WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Views {
        let deferred = pipelines::deferred::DeferredRender::new(&device, &viewport);
        Self {
            eye: eye::Eye::new(device, &deferred, world, viewport, root_node),
            refraction: refraction::Refraction::new(device, &deferred, world, viewport, root_node),
            reflection: reflection::Reflection::new(device, &deferred, world, viewport, root_node),
            shadow: shadow::Shadow::new(device, world, viewport, root_node),
            deferred,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) {
        let view = Matrix4::look_at_rh(viewport.eye, viewport.target, Vector3::unit_y());
        self.deferred.update(queue, viewport, view);

        crossbeam_utils::thread::scope(|scope| {
            let eye = &mut self.eye;
            let reflection = &mut self.reflection;
            let refraction = &mut self.refraction;
            let shadow = &mut self.shadow;
            let deferred = &self.deferred;

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
                shadow.update(device, queue, world, viewport, view, deferred, root_node);
            });
        })
        .unwrap();
    }

    pub fn resize(&mut self, device: &wgpu::Device, world: &WorldData, viewport: &camera::Viewport) {
        self.deferred = pipelines::deferred::DeferredRender::new(&device, &viewport);
        self.eye.resize(device, &self.deferred, world, viewport);
        self.reflection.resize(device, &self.deferred, world, viewport);
        self.refraction.resize(device, &self.deferred, viewport);
        self.shadow.resize(viewport);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, world: &WorldData, target: &wgpu::TextureView) {
        self.shadow.render(encoder, &self.deferred);
        self.reflection.render(encoder, &self.deferred, world);
        self.refraction.render(encoder, &self.deferred, world);
        self.eye.render(encoder, &self.deferred, world, target);
    }
}
