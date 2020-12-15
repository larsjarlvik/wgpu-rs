use std::collections::HashMap;
use wgpu::util::DeviceExt;
use crate::settings;
pub mod data;
mod mesh;
mod render_pipeline;

pub struct Model {
    primitives: Vec<render_pipeline::Primitive>,
    instances: data::InstanceBuffer,
}

pub struct Models {
    render_pipeline: render_pipeline::RenderPipeline,
    models: HashMap<String, Model>,
    sampler: wgpu::Sampler,
    pub render_bundle: wgpu::RenderBundle,
}

impl Models {
    pub fn new(device: &wgpu::Device) -> Self {
        let render_pipeline = render_pipeline::RenderPipeline::new(&device);
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
        let render_bundle = create_bundle(&device, &render_pipeline, &models);

        Self {
            sampler,
            render_pipeline,
            models,
            render_bundle,
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

        let instances = data::InstanceBuffer::new(&device, Vec::<data::Instance>::new());
        self.models.insert(name.to_string(), Model { primitives, instances });
    }

    pub fn add_instance(&mut self, name: &str, instance: data::Instance) {
        let model = self.models.get_mut(&name.to_string()).expect("Model not found!");
        &model.instances.data.push(instance);
    }

    pub fn write_instance_buffers(&mut self, device: &wgpu::Device, name: &str) {
        let model = self.models.get_mut(name).expect("Model not found!");
        model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&model.instances.data),
            usage: wgpu::BufferUsage::VERTEX,
        });
    }

    pub fn refresh_render_bundle(&mut self, device: &wgpu::Device) {
        self.render_bundle = create_bundle(&device, &self.render_pipeline, &self.models);
    }

    pub fn set_uniforms(&mut self, queue: &wgpu::Queue, uniforms: data::Uniforms) {
        self.render_pipeline.uniforms.data = uniforms;
        queue.write_buffer(
            &self.render_pipeline.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.render_pipeline.uniforms.data]),
        );
    }
}

pub fn create_bundle(device: &wgpu::Device, render_pipeline: &render_pipeline::RenderPipeline, models: &HashMap<String, Model>) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });

    encoder.set_pipeline(&render_pipeline.render_pipeline);
    encoder.set_bind_group(1, &render_pipeline.uniforms.bind_group, &[]);

    for model in models.values() {
        for mesh in &model.primitives {
            encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
            encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
            encoder.set_index_buffer(mesh.index_buffer.slice(..));
            encoder.draw_indexed(0..mesh.num_elements, 0, 0..model.instances.data.len() as _);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some("models"),
    })
}