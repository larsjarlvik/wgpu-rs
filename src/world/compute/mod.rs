pub use self::task::Task;
use crate::{noise, settings, texture};
use std::time::Instant;
pub mod task;
mod uniforms;

pub struct Compute {
    pub elevation_pipeline: wgpu::ComputePipeline,
    pub normal_pipeline: wgpu::ComputePipeline,
    pub erosion_pipeline: wgpu::ComputePipeline,
    pub smooth_pipeline: wgpu::ComputePipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    noise_bindings: noise::NoiseBindings,
    compute_texture_bind_group: wgpu::BindGroup,
    elevation_normal_view: wgpu::TextureView,
}

pub struct ComputeBindings {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Compute {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise) -> Self {
        let noise_bindings = noise.create_bindings(device);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    view_dimension: wgpu::TextureViewDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    access: wgpu::StorageTextureAccess::ReadWrite,
                },
                count: None,
            }],
        });

        let size = wgpu::Extent3d {
            width: settings::MAP_DETAIL,
            height: settings::MAP_DETAIL,
            depth: 1,
        };
        let frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::COPY_DST,
        };
        let texture = device.create_texture(frame_descriptor);
        let elevation_normal_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_texture"),
            layout: &texture_bind_group_layout,
            entries: &[texture::create_bind_group_entry(0, &elevation_normal_view)],
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
                &texture_bind_group_layout,
                &uniform_bind_group_layout,
                &noise_bindings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain-elev.comp.spv"));
        let elevation_pipeline = create_pipeline(device, &layout, &module, "elevation");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain-norm.comp.spv"));
        let normal_pipeline = create_pipeline(device, &layout, &module, "normal");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain-erosion.comp.spv"));
        let erosion_pipeline = create_pipeline(device, &layout, &module, "erosion");

        let module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain-smooth.comp.spv"));
        let smooth_pipeline = create_pipeline(device, &layout, &module, "smooth");

        Self {
            elevation_pipeline,
            normal_pipeline,
            erosion_pipeline,
            smooth_pipeline,
            uniform_bind_group_layout,
            noise_bindings,
            compute_texture_bind_group,
            elevation_normal_view,
        }
    }

    pub async fn compute<'a>(&self, device: &wgpu::Device, queue: &wgpu::Queue, tasks: Vec<&'a Task<'a>>) {
        let now = Instant::now();

        tasks.iter().for_each(|task| {
            let now = Instant::now();
            for _ in 0..task.run_times {
                for i in 0..task.stage_count {
                    let uniforms = uniforms::UniformBuffer::new(
                        device,
                        &self.uniform_bind_group_layout,
                        uniforms::Uniforms {
                            size: settings::MAP_DETAIL,
                            octaves: settings::TERRAIN_OCTAVES,
                            sea_level: settings::SEA_LEVEL,
                            horizontal_scale: settings::HORIZONTAL_SCALE,
                            vertical_scale: settings::VERTICAL_SCALE,
                            current_stage: i as u32,
                            stage_count: task.stage_count,
                        },
                    );

                    let encoder = self.run_compute(device, &task, &uniforms.bind_group);
                    queue.submit(std::iter::once(encoder.finish()));
                    device.poll(wgpu::Maintain::Wait);
                }
            }
            println!("{}: {} ms", task.label, now.elapsed().as_millis());
        });

        println!("Total compute: {} ms", now.elapsed().as_millis());
    }

    fn run_compute(&self, device: &wgpu::Device, task: &Task, uniform_bind_group: &wgpu::BindGroup) -> wgpu::CommandEncoder {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(format!("compute_{}", task.label.to_lowercase()).as_str()),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(&task.pipeline);
            pass.set_bind_group(0, &self.compute_texture_bind_group, &[]);
            pass.set_bind_group(1, uniform_bind_group, &[]);
            pass.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
            pass.dispatch(settings::MAP_DETAIL, settings::MAP_DETAIL, 1);
        }
        encoder
    }

    pub fn create_bindings(&self, device: &wgpu::Device) -> ComputeBindings {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_texture"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.elevation_normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        ComputeBindings {
            bind_group_layout,
            bind_group,
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
