use crate::{
    camera::{self, frustum::*},
    settings,
};
use cgmath::*;
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

        let mut models = HashMap::new();
        let render_bundle = create_bundle(&device, &render_pipeline, &camera, &mut models);

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

        let instances = data::InstanceBuffer::new(&device, Vec::new());
        self.models.insert(name.to_string(), Model { primitives, instances });
    }

    pub fn add_instances(&mut self, model_name: &str, instances: Vec<data::Instance>) {
        let model = self.models.get_mut(&model_name.to_string()).expect("Model not found!");
        for instance in instances {
            &model.instances.data.push(instance);
        }
    }

    pub fn refresh_render_bundle(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.render_bundle = create_bundle(&device, &self.render_pipeline, &camera, &mut self.models);
    }
}

pub fn create_bundle(
    device: &wgpu::Device,
    render_pipeline: &render_pipeline::RenderPipeline,
    camera: &camera::Camera,
    models: &mut HashMap<String, Model>,
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

    for (_, model) in models.iter_mut() {
        let length = cull_frustum(device, camera, model);

        for mesh in &model.primitives {
            encoder.set_bind_group(0, &mesh.texture_bind_group, &[]);
            encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, model.instances.buffer.slice(..));
            encoder.set_index_buffer(mesh.index_buffer.slice(..));
            encoder.draw_indexed(0..mesh.num_elements, 0, 0..length as _);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") })
}

fn cull_frustum(device: &wgpu::Device, camera: &camera::Camera, model: &mut Model) -> usize {
    let rad = 8.0; // TODO: Use gltf's bounding box

    let instances = model.instances.data.clone().into_iter().filter(|i| {
        let min = vec3(i.transform[3][0] - rad, i.transform[3][1] - rad, i.transform[3][2] - rad);
        let max = vec3(i.transform[3][0] + rad, i.transform[3][1] + rad, i.transform[3][2] + rad);

        match camera.frustum.test_bounding_box(BoundingBox { min, max }) {
            Intersection::Inside | Intersection::Partial => true,
            Intersection::Outside => false,
        }
    });

    let instances = instances.collect::<Vec<data::Instance>>();
    model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsage::VERTEX,
    });

    instances.len()
}
