use crate::{camera, pipelines, settings};

pub struct Sky {
    pub render_bundle: wgpu::RenderBundle,
}

impl Sky {
    pub fn new(device: &wgpu::Device, camera: &camera::Instance, pipeline: &pipelines::sky::Sky) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: None,
            sample_count: 1,
        });

        encoder.set_pipeline(&pipeline.render_pipeline);
        encoder.set_bind_group(0, &pipeline.texture_bind_group, &[]);
        encoder.set_bind_group(1, &pipeline.uniforms.bind_group, &[]);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("sky") });
        Self { render_bundle }
    }
}
