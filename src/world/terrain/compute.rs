use crate::{noise, plane, settings};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Compute {
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    plane: plane::Plane,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    noise_bindings: noise::NoiseBindings,
}

impl Compute {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise) -> Self {
        let plane = plane::Plane::new(settings::TILE_SIZE);
        let noise_bindings = noise.create_bindings(device);
        let mut lods = vec![];

        for lod in 0..=settings::LODS.len() {
            let indices_lod = plane.create_indices(&device, lod as u32 + 1);
            lods.push(indices_lod);
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
            vertex_bind_group_layout,
            uniform_bind_group_layout,
            noise_bindings,
            plane,
        }
    }

    pub fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: f32, z: f32) -> wgpu::Buffer {
        let contents: &[u8] = bytemuck::cast_slice(&self.plane.vertices);
        let dst_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
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
            pass.dispatch(self.plane.length, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));
        dst_vertex_buffer
    }
}
