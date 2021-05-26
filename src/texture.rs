use std::num::NonZeroU32;
use wgpu::util::DeviceExt;
use wgpu_mipmap::{MipmapGenerator, RecommendedMipmapGenerator};

pub fn create_mipmapped_view(device: &wgpu::Device, queue: &wgpu::Queue, pixels: &Vec<u8>, width: u32, height: u32) -> wgpu::TextureView {
    let size = wgpu::Extent3d { width, height, depth: 1 };

    let generator = RecommendedMipmapGenerator::new(device);
    let texture_descriptor = wgpu::TextureDescriptor {
        size,
        mip_level_count: 9,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
        label: None,
    };

    let texture = device.create_texture(&texture_descriptor);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mipmap_buffer"),
                contents: pixels.as_slice(),
                usage: wgpu::BufferUsage::COPY_SRC,
            }),
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: height,
            },
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        size,
    );

    generator.generate(&device, &mut encoder, &texture, &texture_descriptor).unwrap();
    queue.submit(std::iter::once(encoder.finish()));

    view
}

pub fn create_bind_group_layout(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}
pub fn create_array_bind_group_layout(binding: u32, sample_type: wgpu::TextureSampleType, count: usize) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: Some(NonZeroU32::new(count as u32).unwrap()),
    }
}

pub fn create_view(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> wgpu::TextureView {
    let texture_extent = wgpu::Extent3d { width, height, depth: 1 };
    let frame_descriptor = &wgpu::TextureDescriptor {
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
    };
    let texture = device.create_texture(frame_descriptor);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_sampler(device: &wgpu::Device, address_mode: wgpu::AddressMode, filter_mode: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("sampler"),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: filter_mode,
        ..Default::default()
    })
}

pub fn create_shadow_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("shadow_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    })
}

pub fn create_bind_group_entry(binding: u32, texture_view: &wgpu::TextureView) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(texture_view),
    }
}
