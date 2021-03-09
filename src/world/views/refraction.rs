use crate::{
    camera, deferred,
    world::{node, terrain::bundle::TerrainBundle, WorldData},
};
use cgmath::*;

pub struct Refraction {
    pub terrain: TerrainBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Refraction {
    pub fn new(
        device: &wgpu::Device,
        deferred_render: &deferred::DeferredRender,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, -1.0, 0.0, 1.0]);
        let deferred = deferred_render.get_render_bundle(device, &camera);

        Self {
            terrain: TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node),
            camera,
            deferred,
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
        let view = Matrix4::look_at(viewport.eye, viewport.target, Vector3::unit_y());
        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);
        self.terrain = TerrainBundle::new(device, &self.camera, &mut world_data.terrain, &root_node);
    }

    pub fn resize(&mut self, device: &wgpu::Device, deferred_render: &deferred::DeferredRender, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
        self.deferred = deferred_render.get_render_bundle(device, &self.camera);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData) {
        deferred_render.render_to(encoder, vec![&self.terrain.render_bundle]);
        deferred_render.render(
            encoder,
            &world_data.water.refraction_texture_view,
            &world_data.water.refraction_depth_texture_view,
            &self.deferred,
        );
    }
}
