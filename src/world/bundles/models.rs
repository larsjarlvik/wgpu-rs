use std::borrow::BorrowMut;

use crate::{
    camera::{self, frustum},
    models, pipelines, settings,
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use wgpu::util::DeviceExt;

pub struct ModelsBundle {
    pub render_bundle: wgpu::RenderBundle,
}

impl ModelsBundle {
    pub fn new(device: &wgpu::Device, camera: &camera::Instance, pipeline: &pipelines::model::Model, models: &mut models::Models) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&pipeline.render_pipeline);
        encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

        models.models.borrow_mut().par_iter_mut().for_each(|(_, model)| {
            cull_frustum(device, camera, model);
        });

        for (_, model) in &mut models.models.iter_mut() {
            for mesh in &model.primitives {
                encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..model.instances.visible as _);
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") });
        Self { render_bundle }
    }
}

fn cull_frustum(device: &wgpu::Device, camera: &camera::Instance, model: &mut models::model::Model) {
    let instances = model
        .instances
        .data
        .values()
        .filter(|(_, bounding_box)| match camera.frustum.test_bounding_box(bounding_box) {
            frustum::Intersection::Inside | frustum::Intersection::Partial => true,
            frustum::Intersection::Outside => false,
        })
        .map(|(transform, _)| *transform);

    let instances = instances.collect::<Vec<pipelines::model::data::InstanceData>>();
    model.instances.visible = instances.len();
    model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsage::VERTEX,
    });
}
