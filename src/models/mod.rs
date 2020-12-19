use crate::{camera, settings};
use std::collections::HashMap;
use wgpu::util::DeviceExt;
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
        let render_bundle = create_bundle(&device, &render_pipeline, &camera, &models);

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

        let instances = data::InstanceBuffer::new(&device, HashMap::new());
        self.models.insert(name.to_string(), Model { primitives, instances });
    }

    pub fn add_instance(&mut self, model_name: &str, instance_name: String, instance: data::Instance) {
        let model = self.models.get_mut(&model_name.to_string()).expect("Model not found!");
        &model.instances.data.insert(instance_name, instance);
    }

    pub fn remove_instance(&mut self, model_name: &str, instance_name: &String) {
        let model = self.models.get_mut(&model_name.to_string()).expect("Model not found!");
        &model.instances.data.remove(instance_name);
    }

    pub fn write_instance_buffers(&mut self, device: &wgpu::Device, name: &str) {
        let mut model = self.models.get_mut(name).expect("Model not found!");
        model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance_buffer"),
            contents: bytemuck::cast_slice(&model.instances.data.values().cloned().collect::<Vec<data::Instance>>()),
            usage: wgpu::BufferUsage::VERTEX,
        });
    }

    pub fn refresh_render_bundle(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.render_bundle = create_bundle(&device, &self.render_pipeline, &camera, &self.models);
    }
}

pub fn create_bundle(
    device: &wgpu::Device,
    render_pipeline: &render_pipeline::RenderPipeline,
    camera: &camera::Camera,
    models: &HashMap<String, Model>,
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

    encoder.set_pipeline(&render_pipeline.render_pipeline);
    encoder.set_bind_group(1, &camera.uniforms.bind_group, &[]);

    for model in models.into_iter() {
        for mesh in &model.1.primitives {
            encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
            encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, model.1.instances.buffer.slice(..));
            encoder.set_index_buffer(mesh.index_buffer.slice(..));
            encoder.draw_indexed(0..mesh.num_elements, 0, 0..model.1.instances.data.len() as _);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") })
}
