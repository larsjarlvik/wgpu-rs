use crate::{camera::camera, settings};
use super::Sky;

pub struct SkyBundle {
    render_bundle: wgpu::RenderBundle,
}

impl SkyBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera, sky: &Sky) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: None,
            sample_count: 1,
        });

        encoder.set_pipeline(&sky.render_pipeline);
        encoder.set_bind_group(0, &sky.texture_bind_group, &[]);
        encoder.set_bind_group(1, &sky.uniforms.bind_group, &[]);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("sky") });
        Self {
            render_bundle,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            })
            .execute_bundles(std::iter::once(&self.render_bundle));
    }
}