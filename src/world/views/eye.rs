use cgmath::*;
use crate::{camera, deferred, models::bundle::ModelsBundle, world::{WorldData, node, sky::bundle::SkyBundle, terrain::bundle::TerrainBundle, water::bundle::WaterBundle}};

pub struct Eye {
    pub terrain: TerrainBundle,
    pub water: WaterBundle,
    pub models: ModelsBundle,
    pub sky: SkyBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Eye {
    pub fn new(device: &wgpu::Device, deferred_render: &deferred::DeferredRender, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let deferred = deferred_render.get_render_bundle(device, &camera);

        Self {
            terrain: TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node),
            water: WaterBundle::new(device, &camera, &world_data.water, &root_node),
            models: ModelsBundle::new(device, &camera, &mut world_data.models),
            sky: SkyBundle::new(device, &camera, &world_data.sky),
            camera,
            deferred,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world_data: &mut WorldData, viewport: &camera::Viewport, root_node: &node::Node) {
        let view = Matrix4::look_at(viewport.eye, viewport.target, Vector3::unit_y());
        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);

        self.terrain = TerrainBundle::new(device, &self.camera, &mut world_data.terrain, &root_node);
        self.water = WaterBundle::new(device, &self.camera, &world_data.water, &root_node);
        self.models = ModelsBundle::new(device, &self.camera, &mut world_data.models);
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, viewport: &camera::Viewport) {
        self.camera.resize(viewport.width, viewport.height);
        self.sky = SkyBundle::new(device, &self.camera, &world_data.sky);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, deferred_render: &deferred::DeferredRender, world_data: &WorldData, target: &wgpu::TextureView) {
        deferred_render.render_to(encoder, vec![
            &self.terrain.render_bundle,
            &self.models.render_bundle
        ]);
        deferred_render.render(
            encoder,
            &world_data.sky.texture_view,
            &world_data.sky.depth_texture_view,
            &self.deferred,
        );
        self.water.render(
            encoder,
            &world_data.sky.texture_view,
            &world_data.sky.depth_texture_view,
        );

        self.sky.render(encoder, target);
    }
}
