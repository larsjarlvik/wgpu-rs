use crate::{
    camera::{self, frustum::BoundingBox},
    pipelines, plane, settings,
    world::node::{Node, NodeData},
};
use cgmath::*;

pub struct TerrainBundle {
    pub render_bundle: wgpu::RenderBundle,
}

impl TerrainBundle {
    pub fn new(
        device: &wgpu::Device,
        camera: &camera::Instance,
        pipeline: &pipelines::terrain::Terrain,
        nodes: &Vec<(&Node, &NodeData)>,
    ) -> Self {
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

        let direction = camera.uniforms.data.clip[1];
        let plane = camera.uniforms.data.clip[3];
        let eye = vec3(
            camera.uniforms.data.eye_pos[0],
            camera.uniforms.data.eye_pos[1],
            camera.uniforms.data.eye_pos[2],
        );

        for (node, data) in nodes {
            for lod in 0..=settings::LODS.len() {
                let terrain_lod = data.lods.get(lod).expect("Could not get LOD!");

                if check_clip(direction, plane, &node.bounding_box)
                    && plane::get_lod(eye, vec3(node.x, 0.0, node.z), camera.uniforms.data.z_far) == lod as u32
                {
                    let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                    let lod_buffer = terrain_lod.get(&ct).unwrap();

                    encoder.set_vertex_buffer(0, data.terrain_buffer.slice(..));
                    encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
                }
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") });
        Self { render_bundle }
    }
}

fn check_clip(direction: f32, plane: f32, bounding_box: &BoundingBox) -> bool {
    direction == 0.0 || (direction > 0.0 && bounding_box.max.y >= plane) || (direction <= 0.0 && bounding_box.min.y < plane)
}
