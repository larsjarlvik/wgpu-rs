use crate::{camera, settings};
use std::collections::HashMap;

pub mod data;
mod mesh;
mod render_pipeline;

pub struct InstanceBuffer {
    pub buffer: wgpu::Buffer,
    pub length: u32,
}

pub struct Model {
    primitives: Vec<render_pipeline::Primitive>,
}

pub struct Models {
    render_pipeline: render_pipeline::RenderPipeline,
    models: HashMap<String, Model>,
    sampler: wgpu::Sampler,
}

impl Models {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera) -> Self {
        let render_pipeline = render_pipeline::RenderPipeline::new(&device, &camera);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let models = HashMap::new();

        Self {
            sampler,
            render_pipeline,
            models,
        }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str, path: &str) {
        let (gltf, buffers, images) = gltf::import(format!("./res/models/{}", path)).expect("Failed to import GLTF!");
        let mut primitives: Vec<render_pipeline::Primitive> = vec![];

        for gltf_mesh in gltf.meshes() {
            let meshes = mesh::Mesh::new(&device, &queue, gltf_mesh, &buffers, &images);
            for mesh in meshes {
                &primitives.push(mesh.to_primitive(&device, &self.sampler, &self.render_pipeline));
            }
        }

        self.models.insert(name.to_string(), Model { primitives });
    }

    pub fn get_render_bundle(
        &self,
        device: &wgpu::Device,
        camera: &camera::Camera,
        model_instances: &HashMap<String, InstanceBuffer>,
    ) -> wgpu::RenderBundle {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[
                settings::COLOR_TEXTURE_FORMAT,
                settings::COLOR_TEXTURE_FORMAT,
                settings::COLOR_TEXTURE_FORMAT,
            ],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&self.render_pipeline.render_pipeline);
        encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

        for (model_name, buffer) in model_instances {
            for mesh in &self.models.get(model_name).unwrap().primitives {
                encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_vertex_buffer(1, buffer.buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..));
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..buffer.length as _);
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") })
    }
}
