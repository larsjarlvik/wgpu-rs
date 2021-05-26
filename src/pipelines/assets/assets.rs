use std::collections::HashMap;

use crate::{assets, camera, model, texture};
use wgpu::util::DeviceExt;

pub struct Buffers {
    pub texture_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct ModelBuffer {
    pub primitives: Vec<Buffers>,
    pub bounding_box: camera::BoundingBox,
}

pub struct Assets {
    pub models: HashMap<String, ModelBuffer>,
}

impl Assets {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Self {
        let mut models = HashMap::new();

        for asset in assets::ASSETS {
            let model = model::Model::new(device, queue, &asset.model);

            for mesh in asset.meshes {
                for variant in mesh.variants {
                    let mesh = model
                        .meshes
                        .iter()
                        .filter(|m| m.name == *variant)
                        .nth(0)
                        .expect(format!("Could not load mesh {} from {}", &variant, &asset.model).as_str());

                    let primitives = mesh
                        .primitives
                        .iter()
                        .map(|p| to_buffers(device, sampler, p, &model.materials, texture_bind_group_layout))
                        .collect();

                    models.insert(
                        variant.to_string(),
                        ModelBuffer {
                            primitives,
                            bounding_box: mesh.bounding_box.clone(),
                        },
                    );
                }
            }
        }

        Self { models }
    }
}

fn to_buffers(
    device: &wgpu::Device,
    sampler: &wgpu::Sampler,
    primitive: &model::Primitive,
    materials: &Vec<model::Material>,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> Buffers {
    let mut vertices = vec![];
    for i in 0..primitive.positions.len() {
        vertices.push(super::data::Vertex {
            position: primitive.positions[i],
            normals: primitive.normals[i],
            tangents: primitive.tangents[i],
            tex_coords: primitive.tex_coords[i],
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("asset_vertex_buffer"),
        contents: bytemuck::cast_slice(&vertices.as_slice()),
        usage: wgpu::BufferUsage::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("asset_index_buffer"),
        contents: bytemuck::cast_slice(&primitive.indices.as_slice()),
        usage: wgpu::BufferUsage::INDEX,
    });
    let num_elements = primitive.indices.len() as u32;

    let mut entries = vec![];
    let material = materials.iter().filter(|m| m.index == primitive.material_index).nth(0).unwrap();

    match &material.base_color_texture {
        Some(base_color) => {
            entries.push(texture::create_bind_group_entry(0, &base_color));
        }
        None => (),
    }
    match &material.normal_texture {
        Some(normal) => {
            entries.push(texture::create_bind_group_entry(1, &normal));
        }
        None => (),
    }

    entries.push(wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Sampler(&sampler),
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("asset_texture_layout"),
        layout: texture_bind_group_layout,
        entries: &entries,
    });

    Buffers {
        texture_bind_group,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}
