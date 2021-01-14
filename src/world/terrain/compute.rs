use std::{mem, usize};
use wgpu::util::DeviceExt;

use crate::{noise, settings};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct IndexBuffer {
    pub buffer: wgpu::Buffer,
    pub length: u32,

}

pub struct Compute {
    pub lods: Vec<IndexBuffer>,
    vertex_length: u32,
    vertices: Vec<Vertex>,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    noise_bindings: noise::NoiseBindings,
}

impl Compute {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise) -> Self {
        let vertices = create_vertices(settings::TILE_SIZE);
        let num_elements = vertices.len() as u32;
        let noise_bindings = noise.create_bindings(device);
        let mut lods = vec![];

        for lod in 0..=settings::LODS.len() {
            let indices_lod = create_indices(settings::TILE_SIZE, lod as u32 + 1);
            lods.push(IndexBuffer{
                length: indices_lod.len() as u32,
                buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices_lod.as_slice()),
                    usage: wgpu::BufferUsage::INDEX,
                }),
            });
        }

        let vertex_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("input_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    min_binding_size: None,
                    readonly: true,
                },
                count: None,
            }],
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_compute_layout"),
            bind_group_layouts: &[
                &vertex_bind_group_layout,
                &uniform_bind_group_layout,
                &noise_bindings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/terrain.comp.spv"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("terrain_compute_pipeline"),
            layout: Some(&layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &module,
                entry_point: "main",
            },
        });

        Self {
            lods,
            compute_pipeline,
            vertex_length: num_elements,
            vertex_bind_group_layout,
            uniform_bind_group_layout,
            vertices,
            noise_bindings,
        }
    }

    pub fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: f32, z: f32) -> wgpu::Buffer {
        let contents: &[u8] = bytemuck::cast_slice(&self.vertices);
        let dst_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("output_vertex_buffer"),
            contents,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE,
        });

        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.vertex_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(dst_vertex_buffer.slice(..)),
            }],
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&[x, z]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_terrain"),
        });
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &vertex_bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
            pass.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
            pass.dispatch(self.vertex_length, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));
        dst_vertex_buffer
    }
}

fn create_vertices(tile_size: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let half_tile_size = tile_size as f32 / 2.0;
    let tile_size = tile_size + 1;

    for z in 0..tile_size {
        for x in 0..tile_size {
            vertices.push(Vertex {
                position: [(x as f32) - half_tile_size, 0.0, (z as f32) - half_tile_size],
                normal: [0.0, 0.0, 0.0],
            });
        }
    }

    vertices
}

fn create_indices(tile_size: u32, lod: u32) -> Vec<u32> {
    let mut indices = Vec::new();
    let lod = 2u32.pow(lod as u32) / 2;
    let tile_size = tile_size + 1;

    for z in (0..(tile_size - lod)).step_by(lod as usize) {
        if z > 0 {
            indices.push(z * tile_size);
        }
        for x in (0..tile_size).step_by(lod as usize) {
            indices.push(z * tile_size + x);
            indices.push((z + lod) * tile_size + x);
        }
        if z < tile_size - lod {
            indices.push((z + lod) * tile_size + (tile_size - 1));
        }
    }

    indices
}
