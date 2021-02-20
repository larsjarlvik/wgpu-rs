use super::{node, terrain, water, WorldData};
use crate::{camera, settings};

pub struct BundleGroup {
    pub terrain_bundle: wgpu::RenderBundle,
    pub water_bundle: wgpu::RenderBundle,
    pub models_bundle: wgpu::RenderBundle,
}

impl BundleGroup {
    pub fn new(device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) -> Self {
        let terrain_bundle = get_terrain_bundle(device, &camera, &mut world_data.terrain, &root_node);
        let water_bundle = get_water_bundle(device, &camera, &mut world_data.water, &root_node);
        let models_bundle = world_data.models.get_render_bundle(device, &camera);

        Self {
            terrain_bundle,
            water_bundle,
            models_bundle,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) {
        self.models_bundle = world_data.models.get_render_bundle(device, &camera);
        self.water_bundle = get_water_bundle(device, &camera, &mut world_data.water, &root_node);
        self.terrain_bundle = get_terrain_bundle(device, &camera, &mut world_data.terrain, &root_node);
    }
}

fn get_terrain_bundle(
    device: &wgpu::Device,
    camera: &camera::camera::Camera,
    terrain: &mut terrain::Terrain,
    root_node: &node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&terrain.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &terrain.texture_bind_group, &[]);
    encoder.set_bind_group(2, &terrain.noise_bindings.bind_group, &[]);

    for lod in 0..=settings::LODS.len() {
        let terrain_lod = terrain.compute.lods.get(lod).expect("Could not get LOD!");

        for (terrain, connect_type) in root_node.get_nodes(camera, lod as u32) {
            let lod_buffer = terrain_lod.get(&connect_type).unwrap();
            encoder.set_vertex_buffer(0, terrain.terrain_buffer.slice(..));
            encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn get_water_bundle(
    device: &wgpu::Device,
    camera: &camera::camera::Camera,
    water: &mut water::Water,
    root_node: &node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&water.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &water.uniforms.bind_group, &[]);
    encoder.set_bind_group(2, &water.noise_bindings.bind_group, &[]);
    encoder.set_bind_group(3, &water.texture_bind_group, &[]);

    for lod in 0..=settings::LODS.len() {
        let water_lod = water.lods.get(lod).expect("Could not get LOD!");

        for (water, connect_type) in root_node.get_nodes(camera, lod as u32) {
            let lod_buffer = water_lod.get(&connect_type).unwrap();
            encoder.set_vertex_buffer(0, water.water_buffer.slice(..));
            encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") })
}
