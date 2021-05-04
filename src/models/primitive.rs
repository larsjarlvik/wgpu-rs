use super::material;
use crate::{camera, pipelines, texture};
use cgmath::*;
use wgpu::util::DeviceExt;

pub struct Primitive {
    pub bounding_box: camera::BoundingBox,
    vertices: Vec<pipelines::model::Vertex>,
    indices: Vec<u32>,
    material_index: usize,
}

impl Primitive {
    pub fn new(buffers: &Vec<gltf::buffer::Data>, primitive: &gltf::Primitive) -> Self {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let indices = reader.read_indices().unwrap().into_u32().collect::<Vec<u32>>();
        let positions = reader.read_positions().unwrap().collect::<Vec<[f32; 3]>>();
        let normals = reader.read_normals().unwrap().collect::<Vec<[f32; 3]>>();
        let tangents = reader.read_tangents().unwrap().collect::<Vec<[f32; 4]>>();
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<[f32; 2]>>();

        let mut vertices = vec![];
        for i in 0..positions.len() {
            vertices.push(pipelines::model::Vertex {
                position: positions[i],
                normals: normals[i],
                tangents: tangents[i],
                tex_coords: tex_coords[i],
            });
        }

        let material_index = primitive.material().index().unwrap();

        let bounding_box = camera::BoundingBox {
            min: Point3::from(primitive.bounding_box().min),
            max: Point3::from(primitive.bounding_box().max),
        };

        Self {
            vertices,
            indices,
            material_index,
            bounding_box,
        }
    }

    pub fn to_buffers(
        &self,
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        render_pipeline: &pipelines::model::Model,
        materials: &Vec<material::Material>,
    ) -> super::PrimitiveBuffers {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices.as_slice()),
            usage: wgpu::BufferUsage::INDEX,
        });
        let num_elements = self.indices.len() as u32;

        let mut entries = vec![];
        let material = materials.iter().filter(|m| m.index == self.material_index).nth(0).unwrap();

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
            label: Some("model_textures"),
            layout: &render_pipeline.texture_bind_group_layout,
            entries: &entries,
        });

        super::PrimitiveBuffers {
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}
