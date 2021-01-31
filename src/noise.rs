use cgmath::*;
use std::time::Instant;

pub struct Noise {
    noise: Vec<Vec<f32>>,
    size: wgpu::Extent3d,
    texture_view: wgpu::TextureView,
}

pub struct NoiseBindings {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Noise {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let now = Instant::now();

        let size = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth: 1,
        };
        let byte_size = std::mem::size_of::<f32>() as u32;

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    dimension: wgpu::TextureViewDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    readonly: false,
                },
                count: None,
            }],
        });

        let frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::COPY_DST,
        };
        let texture = device.create_texture(frame_descriptor);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let compute_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("noise_texture"),
            layout: &texture_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("noise_compute_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::include_spirv!("./shaders-compiled/noise.comp.spv"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("terrain_compute_pipeline"),
            layout: Some(&layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &module,
                entry_point: "main",
            },
        });

        // Generate noise
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output_vertex_buffer"),
            size: (size.width * size.height * byte_size) as u64,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("noise") });
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&compute_pipeline);
            pass.set_bind_group(0, &compute_texture_bind_group, &[]);
            pass.dispatch(size.width, size.height, 1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            },
            wgpu::BufferCopyView {
                buffer: &buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: byte_size * size.width,
                    rows_per_image: size.height,
                },
            },
            size,
        );

        queue.submit(std::iter::once(encoder.finish()));
        let noise = read_buffer(device, &buffer, size.width as usize).await;

        println!("Noise: {} ms", now.elapsed().as_millis());
        Self { noise, size, texture_view }
    }

    pub fn random(&self, st: Vector2<f32>) -> f32 {
        let s = vec2(
            (st.x as i32).rem_euclid(self.size.width as i32),
            (st.y as i32).rem_euclid(self.size.height as i32),
        );
        self.noise[s.y as usize][s.x as usize]
    }

    pub fn noise(&self, st: Vector2<f32>) -> f32 {
        let i = vec2(st.x.floor(), st.y.floor());
        let f = st - i;
        let a = self.random(i);
        let b = self.random(i + vec2(1.0, 0.0));
        let c = self.random(i + vec2(0.0, 1.0));
        let d = self.random(i + vec2(1.0, 1.0));

        let u = vec2(f.x * f.x * (3.0 - 2.0 * f.x), f.y * f.y * (3.0 - 2.0 * f.y));
        mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y
    }

    pub fn fbm(&self, mut st: Vector2<f32>, octaves: u32) -> f32 {
        let mut v = 0.0;
        let mut a = 0.5;
        let shift = vec2(100.0, 100.0);

        let rot = Matrix2::new(0.5f32.cos(), 0.5f32.sin(), -0.5f32.sin(), 0.5f32.cos());
        for _ in 0..octaves {
            v += a * self.noise(st);
            st = rot * st * 2.0 + shift;
            a *= 0.5;
        }
        v
    }

    pub fn create_bindings(&self, device: &wgpu::Device) -> NoiseBindings {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("noise_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE | wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE | wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("noise_texture"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        NoiseBindings {
            bind_group_layout,
            bind_group,
        }
    }
}

fn mix(x: f32, y: f32, a: f32) -> f32 {
    let yi = f32::one();
    x * (yi - a) + y * a
}

async fn read_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer, width: usize) -> Vec<Vec<f32>> {
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = buffer_future.await {
        let data = buffer_slice.get_mapped_range();
        unsafe {
            let chunks = data.align_to::<f32>();
            let mut grid = vec![];

            let rows: Vec<&[f32]> = chunks.1.chunks_exact(width).collect();
            for row in rows {
                grid.push(row.to_vec());
            }

            grid
        }
    } else {
        panic!("Failed to generate noise!")
    }
}
