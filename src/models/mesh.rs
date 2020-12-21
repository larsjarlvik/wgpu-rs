use crate::{models::*, texture};
use wgpu::util::DeviceExt;

struct Material {
    base_color_texture: texture::Texture,
}

pub struct Mesh {
    pub name: String,
    vertices: Vec<data::Vertex>,
    indices: Vec<u32>,
    material: Material,
}

impl Mesh {
    pub fn new<'a>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: gltf::Mesh<'_>,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> Vec<Self> {
        let mut meshes = vec![];

        for primitive in mesh.primitives() {
            let name = String::from(mesh.name().unwrap());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let indices = reader.read_indices().unwrap().into_u32().collect::<Vec<_>>();
            let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
            let normals = reader.read_normals().unwrap().collect::<Vec<_>>();
            let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect::<Vec<_>>();

            let mut vertices = vec![];
            for i in 0..positions.len() {
                vertices.push(data::Vertex {
                    position: positions[i],
                    normals: normals[i],
                    tex_coords: tex_coords[i],
                });
            }

            let image = images.iter().nth(0).unwrap();
            let base_color_texture = texture::Texture::new(&device, &queue, &image.pixels, image.width, image.height);
            let material = Material { base_color_texture };

            meshes.push(Mesh {
                name,
                vertices,
                indices,
                material,
            });
        }

        meshes
    }

    pub fn to_primitive(
        &self,
        device: &wgpu::Device,
        sampler: &wgpu::Sampler,
        render_pipeline: &render_pipeline::RenderPipeline,
    ) -> render_pipeline::Primitive {
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Diffuse Texture"),
            layout: &render_pipeline.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.material.base_color_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        render_pipeline::Primitive {
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}
