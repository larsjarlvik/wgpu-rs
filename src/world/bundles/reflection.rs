use crate::{camera, deferred, models::bundle::ModelsBundle, world::{WorldData, node, sky::bundle::SkyBundle, terrain::bundle::TerrainBundle}};

pub struct Reflection {
    pub terrain: TerrainBundle,
    pub models: ModelsBundle,
    pub sky: SkyBundle,
}

impl Reflection {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) -> Self {
        Self {
            terrain: TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node),
            models: ModelsBundle::new(device, &camera, &mut world_data.models),
            sky: SkyBundle::new(device, &camera, &world_data.sky),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) {
        self.terrain = TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node);
        self.models = ModelsBundle::new(device, &camera, &mut world_data.models);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) {
        self.sky = SkyBundle::new(device, &camera, &world_data.sky);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData) {
        deferred_render.render_to(encoder, vec![
            &self.terrain.render_bundle,
            &self.models.render_bundle
        ]);
        deferred_render.render(
            encoder,
            &world_data.sky.texture_view,
            &world_data.sky.depth_texture_view,
            &deferred_render.render_bundle,
        );
        self.sky.render(encoder, &world_data.water.reflection_texture_view);
    }
}
