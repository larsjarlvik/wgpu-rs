use crate::models::*;
use wgpu::util::DeviceExt;
use wgpu_mipmap::*;

struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

struct Material {
    base_color_texture: Texture,
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

            let base_color_texture = load_texture(&device, &queue, images.iter().nth(0).unwrap());
            let material = Material {
                base_color_texture,
            };

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

fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue, image: &gltf::image::Data) -> Texture {
    let size = wgpu::Extent3d {
        width: image.width,
        height: image.height,
        depth: 1,
    };

    let generator = RecommendedMipmapGenerator::new(&device);
    let texture_descriptor = wgpu::TextureDescriptor {
        size,
        mip_level_count: 9,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
        label: None,
    };

    let texture = device.create_texture(&texture_descriptor);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Temporary Buffer"),
                contents: image.pixels.as_slice(),
                usage: wgpu::BufferUsage::COPY_SRC,
            }),
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * image.width,
                rows_per_image: image.height,
            },
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        size,
    );

    generator.generate(&device, &mut encoder, &texture, &texture_descriptor).unwrap();
    queue.submit(std::iter::once(encoder.finish()));

    Texture { texture, view }
}
