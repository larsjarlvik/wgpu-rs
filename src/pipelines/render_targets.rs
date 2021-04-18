use nanoid::nanoid;
use std::collections::HashMap;

pub struct RenderTargets {
    pub targets: HashMap<String, wgpu::TextureView>,
}

impl RenderTargets {
    pub fn new() -> Self {
        Self { targets: HashMap::new() }
    }

    pub fn create(&mut self, device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> String {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = nanoid!();

        self.targets.insert(id.clone(), view);
        id
    }

    pub fn get(&self, target: &String) -> &wgpu::TextureView {
        self.targets.get(target).unwrap()
    }
}
