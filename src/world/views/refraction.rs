use cgmath::*;
use crate::{camera, deferred, world::{WorldData, node, terrain::bundle::TerrainBundle}};

pub struct Refraction {
    pub terrain: TerrainBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Refraction {
    pub fn new(device: &wgpu::Device, deferred_render: &deferred::DeferredRender, world_data: &mut WorldData, cc: &camera::Controller, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &cc, [0.0, -1.0, 0.0, 1.0]);
        let deferred = deferred_render.get_render_bundle(device, &camera);

        Self {
            terrain: TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node),
            camera,
            deferred,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world_data: &mut WorldData, cc: &camera::Controller, root_node: &node::Node) {
        let view = Matrix4::look_at(cc.eye, cc.target, Vector3::unit_y());
        self.camera.update(queue, cc.target, cc.eye, cc.proj * view);
        self.terrain = TerrainBundle::new(device, &self.camera, &mut world_data.terrain, &root_node);
    }

    pub fn resize(&mut self, cc: &camera::Controller) {
        self.camera.uniforms.data.viewport_size = vec2(cc.width as f32, cc.height as f32).into();
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData) {
        deferred_render.render_to(encoder, vec![
            &self.terrain.render_bundle,
        ]);
        deferred_render.render(
            encoder,
            &world_data.water.refraction_texture_view,
            &world_data.water.refraction_depth_texture_view,
            &self.deferred,
        );
    }
}
