use crate::{camera, deferred, world::{WorldData, node, terrain::bundle::TerrainBundle}};

pub struct Refraction {
    pub terrain: TerrainBundle,
}

impl Refraction {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) -> Self {
        Self {
            terrain: TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) {
        self.terrain = TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData) {
        deferred_render.render_to(encoder, vec![
            &self.terrain.render_bundle,
        ]);
        deferred_render.render(
            encoder,
            &world_data.water.refraction_texture_view,
            &world_data.water.refraction_depth_texture_view,
            &deferred_render.render_bundle,
        );
    }
}
