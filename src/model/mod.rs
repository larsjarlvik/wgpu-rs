use std::path::PathBuf;

use rayon::prelude::*;
mod material;
mod mesh;
mod primitive;

pub use self::material::Material;
pub use self::mesh::Mesh;
pub use self::primitive::Primitive;

pub struct Model {
    pub meshes: Vec<mesh::Mesh>,
    pub materials: Vec<material::Material>,
}

impl Model {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, path: &PathBuf) -> Self {
        let (gltf, buffers, images) = gltf::import(path).expect("Failed to import GLTF!");

        let materials = gltf
            .materials()
            .into_iter()
            .par_bridge()
            .map(|material| material::Material::new(&device, &queue, &material, &images))
            .collect();

        let meshes = gltf.meshes().map(|gltf_mesh| mesh::Mesh::new(gltf_mesh, &buffers)).collect();

        Self { meshes, materials }
    }
}
