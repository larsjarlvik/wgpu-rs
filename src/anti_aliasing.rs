use crate::{pipelines, settings, texture};
use smaa::{SmaaMode, SmaaTarget};

pub enum Mode {
    Smaa(SmaaTarget),
    Fxaa(pipelines::fxaa::Fxaa),
    None,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::None
    }
}

#[derive(Default)]
pub struct AntiAliasing {
    pub mode: Mode,
    pub depth_texture_view: Option<wgpu::TextureView>,
    width: u32,
    height: u32,
}

impl AntiAliasing {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
        let depth_texture_view = texture::create_view(device, width, height, settings::DEPTH_TEXTURE_FORMAT);
        Self {
            mode: Mode::Smaa(create_smaa(device, queue, width, height)),
            depth_texture_view: Some(depth_texture_view),
            width,
            height,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_texture_view = Some(texture::create_view(device, width, height, settings::DEPTH_TEXTURE_FORMAT));

        match &mut self.mode {
            Mode::Smaa(smaa) => smaa.resize(device, width, height),
            Mode::Fxaa(fxaa) => fxaa.resize(device, width, height),
            Mode::None => {}
        }
    }

    pub fn execute<F: Fn(&wgpu::TextureView, &wgpu::TextureView)>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        f: F,
    ) {
        if let Some(depth_texture_view) = &self.depth_texture_view {
            match &mut self.mode {
                Mode::Fxaa(fxaa) => {
                    f(&fxaa.texture_view, depth_texture_view);
                }
                Mode::Smaa(smaa) => {
                    f(&smaa.start_frame(device, queue, view), &depth_texture_view);
                }
                Mode::None => {
                    f(&view, &depth_texture_view);
                }
            };

            match &self.mode {
                Mode::Fxaa(fxaa) => {
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
                    fxaa.render(&mut encoder, &view);
                    queue.submit(std::iter::once(encoder.finish()));
                }
                _ => {}
            }
        }
    }

    pub fn toggle(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.mode = match self.mode {
            Mode::Smaa(_) => Mode::Fxaa(create_fxaa(device, self.width, self.height)),
            Mode::Fxaa(_) => Mode::None,
            Mode::None => Mode::Smaa(create_smaa(device, queue, self.width, self.height)),
        }
    }

    pub fn display(&self) -> &str {
        match self.mode {
            Mode::Fxaa(_) => "FXAA",
            Mode::Smaa(_) => "SMAA",
            Mode::None => "None",
        }
    }
}

fn create_smaa(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> SmaaTarget {
    SmaaTarget::new(&device, &queue, width, height, settings::COLOR_TEXTURE_FORMAT, SmaaMode::Smaa1X)
}

fn create_fxaa(device: &wgpu::Device, width: u32, height: u32) -> pipelines::fxaa::Fxaa {
    pipelines::fxaa::Fxaa::new(&device, width, height)
}
