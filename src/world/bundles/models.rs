use wgpu::util::DeviceExt;

use crate::{
    camera, models,
    pipelines::{self, model},
    settings,
    world::node::{Node, NodeData},
};

pub struct Models {
    pub render_bundle: wgpu::RenderBundle,
}

impl Models {
    pub fn new(
        device: &wgpu::Device,
        camera: &camera::Instance,
        pipeline: &pipelines::model::Model,
        models: &mut models::Models,
        nodes: &Vec<(&Node, &NodeData)>,
    ) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&pipeline.render_pipeline);
        encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

        for (key, model) in &mut models.models.iter_mut() {
            let mut instances: Vec<model::Instance> = Vec::new();
            for (_, data) in nodes {
                let node_instances = data.model_instances.get(key).expect("Could not find model instance!");
                instances.extend(node_instances);
            }

            model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsage::VERTEX,
            });
            for mesh in &model.primitives {
                encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..instances.len() as _);
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") });
        Self { render_bundle }
    }

    pub fn new_shadow_bundle(
        device: &wgpu::Device,
        camera: &camera::Instance,
        pipeline: &pipelines::model::Model,
        models: &mut models::Models,
        nodes: &Vec<(&Node, &NodeData)>,
    ) -> Self {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&pipeline.shadow_pipeline);
        encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

        for (key, model) in &mut models.models.iter_mut() {
            let mut instances: Vec<model::Instance> = Vec::new();
            for (_, data) in nodes {
                let node_instances = data.model_instances.get(key).expect("Could not find model instance!");
                instances.extend(node_instances);
            }

            model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsage::VERTEX,
            });
            for mesh in &model.primitives {
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..instances.len() as _);
            }
        }

        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("models_shadow"),
        });
        Self { render_bundle }
    }
}
