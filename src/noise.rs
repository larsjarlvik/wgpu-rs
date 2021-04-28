use crate::{settings, texture};
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::time::Instant;

pub struct Noise {
    texture_view: wgpu::TextureView,
}

pub struct NoiseBindings {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Noise {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let now = Instant::now();
        let size = 512;
        let texture_size = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };

        let frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_SRC | wgpu::TextureUsage::COPY_DST,
        };
        let texture = device.create_texture(frame_descriptor);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut rng: Pcg64 = Seeder::from(settings::MAP_SEED).make_rng();
        let mut noise_data = vec![];
        for _ in 0..size {
            for _ in 0..size {
                let num: f32 = rng.gen();
                noise_data.push(num);
            }
        }

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytemuck::cast_slice(&noise_data),
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * size as u32,
                rows_per_image: size as u32,
            },
            texture_size,
        );

        println!("Noise: {} ms", now.elapsed().as_millis());
        Self { texture_view }
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
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE | wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("noise_texture"),
            layout: &bind_group_layout,
            entries: &[
                texture::create_bind_group_entry(0, &self.texture_view),
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
