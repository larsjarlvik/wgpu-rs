use super::{node, WorldData};
use crate::{
    camera,
    pipelines::{self, render_targets},
};
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
    pub fn new(
        device: &wgpu::Device,
        world: &WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
        render_targets: &mut render_targets::RenderTargets,
    ) -> Views {
        let deferred = pipelines::deferred::DeferredRender::new(&device, &viewport, render_targets);
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

        self.eye.update(device, queue, world, viewport, view, root_node);
        self.reflection.update(device, queue, world, viewport, root_node);
        self.refraction.update(device, queue, world, viewport, root_node);
        self.shadow.update(device, queue, world, viewport, view, &self.deferred, root_node);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        world: &WorldData,
        viewport: &camera::Viewport,
        render_targets: &mut render_targets::RenderTargets,
    ) {
        self.deferred = pipelines::deferred::DeferredRender::new(&device, &viewport, render_targets);
        self.eye.resize(device, &self.deferred, world, viewport);
        self.reflection.resize(device, &self.deferred, world, viewport);
        self.refraction.resize(device, &self.deferred, viewport);
        self.shadow.resize(viewport);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        world: &WorldData,
        render_targets: &render_targets::RenderTargets,
        target: &wgpu::TextureView,
    ) {
        self.shadow.render(encoder, &self.deferred, render_targets);
        self.reflection.render(encoder, &self.deferred, world, render_targets);
        self.refraction.render(encoder, &self.deferred, world, render_targets);
        self.eye.render(encoder, &self.deferred, world, render_targets, target);
    }
}
