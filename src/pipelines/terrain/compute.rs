use crate::{
    noise,
    plane::{self, Vertex},
    settings,
};
use futures::executor::block_on;
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
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
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

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/terrain.comp.spv"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("terrain_compute_pipeline"),
            layout: Some(&layout),
            module: &module,
            entry_point: "main",
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

    pub fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: f32, z: f32) -> (f32, f32, wgpu::Buffer) {
        let contents: &[u8] = bytemuck::cast_slice(&self.plane.vertices);
        let dst_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
        });

        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.vertex_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: dst_vertex_buffer.as_entire_binding(),
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
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_terrain"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &vertex_bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
            pass.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
            pass.dispatch(self.plane.length, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));

        let (y_min, y_max) = block_on(get_elevation_range(device, &dst_vertex_buffer));

        dst_vertex_buffer.unmap();
        (y_min, y_max, dst_vertex_buffer)
    }
}

async fn get_elevation_range(device: &wgpu::Device, buffer: &wgpu::Buffer) -> (f32, f32) {
    let buffer_slice = buffer.clone().slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = buffer_future.await {
        let data = buffer_slice.get_mapped_range();
        unsafe {
            let chunks = data.align_to::<Vertex>();
            let mut min: f32 = chunks.1.first().unwrap().position[1];
            let mut max: f32 = chunks.1.first().unwrap().position[1];
            for vertex in chunks.1.iter() {
                min = min.min(vertex.position[1]);
                max = max.max(vertex.position[1]);
            }
            (min, max)
        }
    } else {
        panic!("Failed to generate noise!")
    }
}
