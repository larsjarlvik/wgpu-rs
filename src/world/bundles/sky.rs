use crate::{camera, pipelines, settings};

pub fn get_sky_bundle(device: &wgpu::Device, camera: &camera::Instance, pipeline: &pipelines::sky::Sky) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: None,
        sample_count: 1,
    });

    encoder.set_pipeline(&pipeline.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &pipeline.uniforms.bind_group, &[]);
    encoder.draw(0..6, 0..1);

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("sky") })
}
