use crate::{camera::camera, settings, world::node};
use super::Water;

pub struct WaterBundle {
    render_bundle: wgpu::RenderBundle,
}

impl WaterBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera, water: &Water, root_node: &node::Node) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("water_bundle"),
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

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") });
        Self {
            render_bundle,
        }
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