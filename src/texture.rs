use wgpu::util::DeviceExt;
use wgpu_mipmap::{MipmapGenerator, RecommendedMipmapGenerator};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, pixels: &Vec<u8>, width: u32, height: u32) -> Texture {
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };

        let generator = RecommendedMipmapGenerator::new(&device);
        let texture_descriptor = wgpu::TextureDescriptor {
            size,
            mip_level_count: 9,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
            label: None,
        };

        let texture = device.create_texture(&texture_descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Temporary Buffer"),
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

        Texture { texture, view }
    }
}
