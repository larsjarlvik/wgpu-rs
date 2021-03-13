use crate::{camera, pipelines, plane, settings, world::node};
use cgmath::*;

pub struct WaterBundle {
    render_bundle: wgpu::RenderBundle,
}

impl WaterBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Instance, pipeline: &pipelines::water::Water, root_node: &node::Node) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("water_bundle"),
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });
        encoder.set_pipeline(&pipeline.render_pipeline);
        encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(1, &pipeline.uniforms.bind_group, &[]);
        encoder.set_bind_group(2, &pipeline.noise_bindings.bind_group, &[]);
        encoder.set_bind_group(3, &pipeline.texture_bind_group, &[]);

        for lod in 0..=settings::LODS.len() {
            let water_lod = pipeline.lods.get(lod).expect("Could not get LOD!");

            for node in root_node.get_nodes(camera) {
                let plane = camera.uniforms.data.clip[3];
                let eye = vec3(
                    camera.uniforms.data.eye_pos[0],
                    camera.uniforms.data.eye_pos[1],
                    camera.uniforms.data.eye_pos[2],
                );

                if check_clip(plane, node.bounding_box.min.y) {
                    let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                    let lod_buffer = water_lod.get(&ct).unwrap();

                    match &node.data {
                        Some(data) => {
                            encoder.set_vertex_buffer(0, data.water_buffer.slice(..));
                            encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
                        }
                        None => {}
                    }
                }
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") });
        Self { render_bundle }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, depth_target: &wgpu::TextureView) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("water_render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(std::iter::once(&self.render_bundle));
    }
}

fn check_clip(plane: f32, y_min: f32) -> bool {
    y_min <= plane
}
