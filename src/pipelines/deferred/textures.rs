use crate::{settings, texture};

pub struct Textures {
    pub normals_texture_view: wgpu::TextureView,
    pub base_color_texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
    pub shadow_texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Textures {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let sampler = texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Nearest);
        let normals_texture_view = texture::create_view(&device, width, height, settings::COLOR_TEXTURE_FORMAT);
        let base_color_texture_view = texture::create_view(&device, width, height, settings::COLOR_TEXTURE_FORMAT);
        let depth_texture_view = texture::create_view(&device, width, height, settings::DEPTH_TEXTURE_FORMAT);
        let shadow_texture_view = texture::create_view(
            &device,
            settings::SHADOW_RESOLUTION,
            settings::SHADOW_RESOLUTION,
            settings::DEPTH_TEXTURE_FORMAT,
        );

        Self {
            normals_texture_view,
            base_color_texture_view,
            depth_texture_view,
            shadow_texture_view,
            sampler,
        }
    }

    pub fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("deferred_textures_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Depth),
                texture::create_bind_group_layout(1, wgpu::TextureSampleType::Uint),
                texture::create_bind_group_layout(2, wgpu::TextureSampleType::Uint),
                texture::create_bind_group_layout(3, wgpu::TextureSampleType::Depth),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred_textures"),
            layout: &layout,
            entries: &[
                texture::create_bind_group_entry(0, &self.depth_texture_view),
                texture::create_bind_group_entry(1, &self.normals_texture_view),
                texture::create_bind_group_entry(2, &self.base_color_texture_view),
                texture::create_bind_group_entry(3, &self.shadow_texture_view),
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}
