use cgmath::*;

use crate::texture;

pub struct Textures {
    pub elevation_normal: wgpu::TextureView,
    pub elevation_normal_texture: wgpu::Texture,
    pub biome: wgpu::TextureView,
    pub biome_texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Textures {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let texture_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::COPY_DST,
        };

        let elevation_normal_texture = device.create_texture(texture_descriptor);
        let biome_texture = device.create_texture(texture_descriptor);

        let elevation_normal = elevation_normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let biome = biome_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Nearest);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Float { filterable: true }),
                texture::create_bind_group_layout(1, wgpu::TextureSampleType::Float { filterable: true }),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
                texture::create_bind_group_entry(0, &elevation_normal),
                texture::create_bind_group_entry(1, &biome),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            elevation_normal,
            elevation_normal_texture,
            biome,
            biome_texture,
            bind_group_layout,
            bind_group,
        }
    }

    pub async fn read_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, size: u32) -> Vec<Vector4<f32>> {
        let f32_size = std::mem::size_of::<f32>() as f32;

        let output_buffer_size = (f32_size * size as f32 * size as f32 * 4.0) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &output_buffer,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: f32_size as u32 * size * 4,
                    rows_per_image: size,
                },
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);
        mapping.await.unwrap();

        let data = buffer_slice.get_mapped_range();
        let buffer = unsafe { data.align_to::<Vector4<f32>>().1 };
        buffer.to_vec()
    }
}
