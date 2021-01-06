use crate::{camera::frustum, *};
use cgmath::*;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
pub mod data;
mod mesh;
mod model;
mod render_pipeline;

pub struct Models {
    pub models: HashMap<String, model::Model>,
    pub render_bundle: wgpu::RenderBundle,
    render_pipeline: render_pipeline::RenderPipeline,
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
        let mut primitives: Vec<render_pipeline::PrimitiveBuffers> = vec![];
        let mut bounding_box = frustum::BoundingBox {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(0.0, 0.0, 0.0),
        };

        for gltf_mesh in gltf.meshes() {
            let mesh = mesh::Mesh::new(&device, &queue, gltf_mesh, &buffers, &images);

            for primitive in mesh.primitives {
                if primitive.bounding_box.min[0] < bounding_box.min.x {
                    bounding_box.min.x = primitive.bounding_box.min[0];
                }
                if primitive.bounding_box.max[0] > bounding_box.max.x {
                    bounding_box.max.x = primitive.bounding_box.max[0];
                }
                if primitive.bounding_box.min[1] < bounding_box.min.y {
                    bounding_box.min.y = primitive.bounding_box.min[1];
                }
                if primitive.bounding_box.max[1] > bounding_box.max.y {
                    bounding_box.max.y = primitive.bounding_box.max[1];
                }
                if primitive.bounding_box.min[2] < bounding_box.min.z {
                    bounding_box.min.z = primitive.bounding_box.min[2];
                }
                if primitive.bounding_box.max[2] > bounding_box.max.z {
                    bounding_box.max.z = primitive.bounding_box.max[2];
                }

                &primitives.push(primitive.to_buffers(&device, &self.sampler, &self.render_pipeline));
            }
        }

        let instances = data::InstanceBuffer::new(&device, Vec::new());
        self.models.insert(
            name.to_string(),
            model::Model {
                primitives,
                instances,
                bounding_box,
            },
        );
    }

    pub fn refresh_render_bundle(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.render_bundle = create_bundle(&device, &self.render_pipeline, &camera, &mut self.models);
    }
}

pub fn create_bundle(
    device: &wgpu::Device,
    render_pipeline: &render_pipeline::RenderPipeline,
    camera: &camera::Camera,
    models: &mut HashMap<String, model::Model>,
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

fn cull_frustum(device: &wgpu::Device, camera: &camera::Camera, model: &mut model::Model) -> usize {
    let instances = model
        .instances
        .data
        .clone()
        .into_iter()
        .filter(|(_, bounding_box)| match camera.frustum.test_bounding_box(bounding_box) {
            frustum::Intersection::Inside | frustum::Intersection::Partial => true,
            frustum::Intersection::Outside => false,
        })
        .map(|(transform, _)| transform);

    let instances = instances.collect::<Vec<data::InstanceData>>();
    model.instances.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsage::VERTEX,
    });

    instances.len()
}
