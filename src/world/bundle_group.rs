use super::{WorldData, node, sky::bundle::SkyBundle, terrain::bundle::TerrainBundle, water::bundle::WaterBundle};
use crate::{camera, models::bundle::ModelsBundle};

pub struct BundleGroup {
    pub terrain: Option<TerrainBundle>,
    pub water: Option<WaterBundle>,
    pub models: Option<ModelsBundle>,
    pub sky: Option<SkyBundle>,
}

impl BundleGroup {
    pub fn new() -> Self {
        Self {
            terrain: None,
            water: None,
            models: None,
            sky: None,
        }
    }

    pub fn with_terrain_bundle(
        mut self,
        device: &wgpu::Device,
        world_data: &mut WorldData,
        camera: &camera::camera::Camera,
        root_node: &node::Node,
    ) -> Self {
        self.terrain = Some(TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node));
        self
    }

    pub fn with_water_bundle(
        mut self,
        device: &wgpu::Device,
        world_data: &mut WorldData,
        camera: &camera::camera::Camera,
        root_node: &node::Node,
    ) -> Self {
        self.water = Some(WaterBundle::new(device, &camera, &world_data.water, &root_node));
        self
    }

    pub fn with_models_bundle(mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) -> Self {
        self.models = Some(ModelsBundle::new(device, &camera, &mut world_data.models));
        self
    }

    pub fn with_sky_bundle(mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) -> Self {
        self.sky = Some(SkyBundle::new(device, &camera, &world_data.sky));
        self
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) {
        if self.terrain.is_some() {
            self.terrain = Some(TerrainBundle::new(device, &camera, &mut world_data.terrain, &root_node));
        }
        if self.water.is_some() {
            self.water = Some(WaterBundle::new(device, &camera, &world_data.water, &root_node));
        }
        if self.models.is_some() {
            self.models = Some(ModelsBundle::new(device, &camera, &mut world_data.models));
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) {
        if self.sky.is_some() {
            self.sky = Some(SkyBundle::new(device, &camera, &world_data.sky));
        }
    }
}
