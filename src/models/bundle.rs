use wgpu::util::DeviceExt;
use crate::{camera::{camera, frustum}, settings};
use super::{Models, data, model};

pub struct ModelsBundle {
    pub render_bundle: wgpu::RenderBundle,
}

impl ModelsBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera, models: &mut Models) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&models.render_pipeline.render_pipeline);
        encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

        for (_, model) in &mut models.models.iter_mut() {
            let length = cull_frustum(device, camera, model);

            for mesh in &model.primitives {
                encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..length as _);
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") });
        Self {
            render_bundle,
        }
    }
}

fn cull_frustum(device: &wgpu::Device, camera: &camera::Camera, model: &mut model::Model) -> usize {
    let instances = model
        .instances
        .data
        .values()
        .clone()
        .into_iter()
        .filter(|(_, bounding_box)| match camera.frustum.test_bounding_box(bounding_box) {
            frustum::Intersection::Inside | frustum::Intersection::Partial => true,
            frustum::Intersection::Outside => false,
        })
        .map(|(transform, _)| *transform);

    let instances = instances.collect::<Vec<data::InstanceData>>();
    model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsage::VERTEX,
    });

    instances.len()
}
