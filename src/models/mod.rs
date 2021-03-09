use crate::{camera::frustum, *};
use cgmath::*;
use std::collections::HashMap;
pub mod data;
mod mesh;
mod model;
mod render_pipeline;
pub mod bundle;

pub struct Models {
    pub models: HashMap<String, model::Model>,
    render_pipeline: render_pipeline::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl Models {
    pub fn new(device: &wgpu::Device, viewport: &camera::Viewport) -> Self {
        let render_pipeline = render_pipeline::RenderPipeline::new(&device, &viewport);
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

        let instances = data::InstanceBuffer::new(&device, HashMap::new());
        self.models.insert(
            name.to_string(),
            model::Model {
                primitives,
                instances,
                bounding_box,
            },
        );
    }
}
