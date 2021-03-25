use super::uniforms;
use crate::{noise, plane, settings};
use plane::Vertex;
use std::time::Instant;
use wgpu::util::DeviceExt;

pub struct Compute {
    pub elevation_pipeline: wgpu::ComputePipeline,
    pub normal_pipeline: wgpu::ComputePipeline,
    pub erosion_pipeline: wgpu::ComputePipeline,
    pub smooth_pipeline: wgpu::ComputePipeline,
    vertex_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    noise_bindings: noise::NoiseBindings,
}

impl Compute {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise) -> Self {
        let noise_bindings = noise.create_bindings(device);

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

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/terrain-elev.comp.spv"));
        let elevation_pipeline = create_pipeline(device, &layout, &module, "elevation");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/terrain-norm.comp.spv"));
        let normal_pipeline = create_pipeline(device, &layout, &module, "normal");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/terrain-erosion.comp.spv"));
        let erosion_pipeline = create_pipeline(device, &layout, &module, "erosion");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/terrain-smooth.comp.spv"));
        let smooth_pipeline = create_pipeline(device, &layout, &module, "smooth");

        Self {
            elevation_pipeline,
            normal_pipeline,
            erosion_pipeline,
            smooth_pipeline,
            vertex_bind_group_layout,
            uniform_bind_group_layout,
            noise_bindings,
        }
    }

    pub async fn compute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipelines: Vec<&wgpu::ComputePipeline>,
        plane: &plane::Plane,
    ) -> plane::Plane {
        let now = Instant::now();
        let contents: &[u8] = bytemuck::cast_slice(&plane.vertices);

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

        let uniforms = uniforms::UniformBuffer::new(
            device,
            &self.uniform_bind_group_layout,
            uniforms::Uniforms {
                size: plane.size + 1,
                octaves: settings::TERRAIN_OCTAVES,
                sea_level: settings::SEA_LEVEL,
                horizontal_scale: settings::HORIZONTAL_SCALE,
                vertical_scale: settings::VERTICAL_SCALE,
            },
        );

        let encoders: Vec<wgpu::CommandBuffer> = pipelines
            .iter()
            .map(|&p| {
                self.run_compute(device, p, &vertex_bind_group, &uniforms.bind_group, plane.length)
                    .finish()
            })
            .collect();

        queue.submit(encoders);
        let plane = self.get_result(device, &dst_vertex_buffer, plane.size).await;
        dst_vertex_buffer.unmap();

        println!("Compute: {} ms", now.elapsed().as_millis());
        plane
    }

    fn run_compute(
        &self,
        device: &wgpu::Device,
        compute_pipeline: &wgpu::ComputePipeline,
        vertex_bind_group: &wgpu::BindGroup,
        uniform_bind_group: &wgpu::BindGroup,
        length: u32,
    ) -> wgpu::CommandEncoder {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_terrain"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&compute_pipeline);
            pass.set_bind_group(0, &vertex_bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
            pass.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
            pass.dispatch(length, 1, 1);
        }
        encoder
    }

    async fn get_result(&self, device: &wgpu::Device, buffer: &wgpu::Buffer, size: u32) -> plane::Plane {
        let buffer_slice = buffer.clone().slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = buffer_future.await {
            let data = buffer_slice.get_mapped_range();

            let vertices = unsafe { data.align_to::<Vertex>().1 };
            let data = plane::from_data(vertices.to_vec(), size);
            data
        } else {
            panic!("Failed to generate terrain!")
        }
    }
}

fn create_pipeline(device: &wgpu::Device, layout: &wgpu::PipelineLayout, module: &wgpu::ShaderModule, name: &str) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(format!("terrain_compute_{}_pipeline", name).as_str()),
        layout: Some(&layout),
        module,
        entry_point: "main",
    })
}
