use crate::{
    camera, pipelines,
    world::{bundles, node, WorldData},
};
use cgmath::*;

pub struct Reflection {
    pub terrain: bundles::terrain::TerrainBundle,
    pub models: bundles::models::ModelsBundle,
    pub sky: bundles::sky::SkyBundle,
    pub camera: camera::Instance,
    pub deferred: wgpu::RenderBundle,
}

impl Reflection {
    pub fn new(
        device: &wgpu::Device,
        deferred_render: &pipelines::deferred::DeferredRender,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
        root_node: &node::Node,
    ) -> Self {
        let camera = camera::Instance::from_controller(device, &viewport, [0.0, 1.0, 0.0, 1.0]);
        let deferred = deferred_render.get_render_bundle(device, &camera);
        let nodes = root_node.get_nodes(&camera);

        Self {
            terrain: bundles::terrain::TerrainBundle::new(device, &camera, &mut world_data.terrain, &nodes),
            models: bundles::models::ModelsBundle::new(device, &camera, &world_data.model, &mut world_data.models, &nodes),
            sky: bundles::sky::SkyBundle::new(device, &camera, &world_data.sky),
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
        let nodes = root_node.get_nodes(&self.camera);
        let view = Matrix4::look_at(
            Point3::new(viewport.eye.x, -viewport.eye.y, viewport.eye.z),
            viewport.target,
            -Vector3::unit_y(),
        );
        self.camera.update(queue, viewport.target, viewport.eye, viewport.proj * view);

        self.terrain = bundles::terrain::TerrainBundle::new(device, &self.camera, &mut world_data.terrain, &nodes);
        self.models = bundles::models::ModelsBundle::new(device, &self.camera, &world_data.model, &mut world_data.models, &nodes);
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        deferred_render: &pipelines::deferred::DeferredRender,
        world_data: &mut WorldData,
        viewport: &camera::Viewport,
    ) {
        self.camera.resize(viewport.width, viewport.height);
        self.deferred = deferred_render.get_render_bundle(device, &self.camera);
        self.sky = bundles::sky::SkyBundle::new(device, &self.camera, &world_data.sky);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        deferred_render: &pipelines::deferred::DeferredRender,
        world_data: &WorldData,
    ) {
        deferred_render.render_to(encoder, vec![&self.terrain.render_bundle, &self.models.render_bundle]);
        deferred_render.render(
            encoder,
            &world_data.sky.texture_view,
            &world_data.sky.depth_texture_view,
            &self.deferred,
        );
        self.sky.render(encoder, &world_data.water.reflection_texture_view);
    }
}
