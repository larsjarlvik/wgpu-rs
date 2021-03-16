use crate::{
    camera, pipelines,
    world::{bundles, node, WorldData},
};
use cgmath::*;
use pipelines::*;

pub struct Eye {
    pub terrain: bundles::TerrainBundle,
    pub water: bundles::WaterBundle,
    pub models: bundles::ModelsBundle,
    pub sky: bundles::SkyBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Eye {
    pub fn new(
        device: &wgpu::Device,
        deferred_render: &deferred::DeferredRender,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let deferred = deferred_render.get_render_bundle(device, &camera);
        let nodes = root_node.get_nodes(&camera);

        Self {
            terrain: bundles::TerrainBundle::new(device, &camera, &mut world_data.terrain, &nodes),
            water: bundles::WaterBundle::new(device, &camera, &world_data.water, &nodes),
            models: bundles::ModelsBundle::new(device, &camera, &world_data.model, &mut world_data.models, &nodes),
            sky: bundles::SkyBundle::new(device, &camera, &world_data.sky),
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
        let nodes = root_node.get_nodes(&self.camera);

        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);
        self.terrain = bundles::TerrainBundle::new(device, &self.camera, &mut world_data.terrain, &nodes);
        self.water = bundles::WaterBundle::new(device, &self.camera, &world_data.water, &nodes);
        self.models = bundles::ModelsBundle::new(device, &self.camera, &world_data.model, &mut world_data.models, &nodes);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        deferred_render: &deferred::DeferredRender,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
    ) {
        self.camera.resize(viewport.width, viewport.height);
        self.deferred = deferred_render.get_render_bundle(device, &self.camera);
        self.sky = bundles::SkyBundle::new(device, &self.camera, &world_data.sky);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        deferred_render: &deferred::DeferredRender,
        world_data: &WorldData,
        target: &wgpu::TextureView,
    ) {
        deferred_render.render_to(encoder, vec![&self.terrain.render_bundle, &self.models.render_bundle]);
        deferred_render.render(
            encoder,
            &world_data.sky.texture_view,
            &world_data.sky.depth_texture_view,
            &self.deferred,
        );
        self.water
            .render(encoder, &world_data.sky.texture_view, &world_data.sky.depth_texture_view);

        self.sky.render(encoder, target);
    }
}
