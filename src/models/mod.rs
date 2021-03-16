use crate::{camera, pipelines};
use cgmath::*;
use model::PrimitiveBuffers;
use std::collections::HashMap;
mod mesh;
pub mod model;
mod primitive;

pub struct Models {
    pub models: HashMap<String, model::Model>,
}

impl Models {
    pub fn new() -> Self {
        let models = HashMap::new();
        Self { models }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &pipelines::model::Model, name: &str, path: &str) {
        let (gltf, buffers, images) = gltf::import(format!("./res/models/{}", path)).expect("Failed to import GLTF!");
        let mut primitives: Vec<PrimitiveBuffers> = vec![];
        let mut bounding_box = camera::BoundingBox {
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

                &primitives.push(primitive.to_buffers(&device, &pipeline.sampler, &pipeline));
            }
        }

        let instances = pipelines::model::InstanceBuffer::new(&device);
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
