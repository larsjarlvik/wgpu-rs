use crate::{assets, camera, pipelines};
use cgmath::*;
use model::PrimitiveBuffers;
use rayon::prelude::*;
use std::collections::HashMap;
mod material;
mod mesh;
mod model;
mod primitive;

pub struct Models {
    pub meshes: HashMap<String, model::Model>,
}

impl Models {
    pub fn new() -> Self {
        let meshes = HashMap::new();
        Self { meshes }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, pipeline: &pipelines::model::Model, asset: &assets::Asset) {
        let (gltf, buffers, images) = gltf::import(format!("./res/assets/{}.glb", asset.model)).expect("Failed to import GLTF!");

        let materials = gltf
            .materials()
            .into_iter()
            .par_bridge()
            .map(|material| material::Material::new(&device, &queue, &material, &images))
            .collect();

        for mesh in asset.meshes {
            let gltf_mesh = gltf
                .meshes()
                .filter(|m| m.name().unwrap() == mesh.name)
                .nth(0)
                .expect(format!("Could not load mesh {} from {}", &mesh.name, &asset.model).as_str());

            let gltf_mesh = mesh::Mesh::new(gltf_mesh, &buffers);
            let mut primitives: Vec<PrimitiveBuffers> = vec![];
            let mut bounding_box = camera::BoundingBox {
                min: Point3::new(0.0, 0.0, 0.0),
                max: Point3::new(0.0, 0.0, 0.0),
            };

            for primitive in gltf_mesh.primitives {
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
                &primitives.push(primitive.to_buffers(&device, &pipeline.sampler, &pipeline, &materials));
            }

            self.meshes.insert(mesh.name.to_string(), model::Model { primitives, bounding_box });
        }
    }
}
