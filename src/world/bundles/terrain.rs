use crate::{camera, pipelines, settings, world::node};

pub struct TerrainBundle {
    pub render_bundle: wgpu::RenderBundle,
}

impl TerrainBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Instance, pipeline: &pipelines::terrain::Terrain, root_node: &node::Node) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("terrain_bundle"),
            color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });
        encoder.set_pipeline(&pipeline.render_pipeline);
        encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(1, &pipeline.texture_bind_group, &[]);
        encoder.set_bind_group(2, &pipeline.noise_bindings.bind_group, &[]);

        for lod in 0..=settings::LODS.len() {
            let terrain_lod = pipeline.compute.lods.get(lod).expect("Could not get LOD!");

            for (terrain, connect_type) in root_node.get_nodes(camera, lod as u32, &check_clip) {
                let lod_buffer = terrain_lod.get(&connect_type).unwrap();
                encoder.set_vertex_buffer(0, terrain.terrain_buffer.slice(..));
                encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") });
        Self { render_bundle }
    }
}

fn check_clip(direction: f32, plane: f32, y_min: f32, y_max: f32) -> bool {
    direction == 0.0 || (direction > 0.0 && y_max >= plane) || (direction <= 0.0 && y_min < plane)
}
