use crate::settings;

pub struct Textures {
    pub normals_texture_view: wgpu::TextureView,
    pub base_color_texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
}

impl Textures {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let normals_texture_view = create_texture_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let base_color_texture_view = create_texture_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let depth_texture_view = create_texture_view(&device, &swap_chain_desc, settings::DEPTH_TEXTURE_FORMAT);

        Self {
            normals_texture_view,
            base_color_texture_view,
            depth_texture_view,
        }
    }

    pub fn create_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("deferred_textures_bind_group_layout"),
            entries: &[
                create_bind_group_layout(0, wgpu::TextureComponentType::Float),
                create_bind_group_layout(1, wgpu::TextureComponentType::Uint),
                create_bind_group_layout(2, wgpu::TextureComponentType::Uint),
            ],
        })
    }

    pub fn create_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred_textures"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.depth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.normals_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.base_color_texture_view),
                },
            ],
        })
    }
}

fn create_bind_group_layout(binding: u32, component_type: wgpu::TextureComponentType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            dimension: wgpu::TextureViewDimension::D2,
            component_type,
        },
        count: None,
    }
}

fn create_texture_view(
    device: &wgpu::Device,
    swap_chain_desc: &wgpu::SwapChainDescriptor,
    format: wgpu::TextureFormat,
) -> wgpu::TextureView {
    let texture_extent = wgpu::Extent3d {
        width: swap_chain_desc.width,
        height: swap_chain_desc.height,
        depth: 1,
    };
    let frame_descriptor = &wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
    };
    let texture = device.create_texture(frame_descriptor);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
