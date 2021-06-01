use crate::{camera, pipelines, settings, texture};
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
}

impl AntiAliasing {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Self {
        let depth_texture_view = texture::create_view(device, viewport.width, viewport.height, settings::DEPTH_TEXTURE_FORMAT);
        Self {
            mode: Mode::Smaa(create_smaa(device, queue, viewport)),
            depth_texture_view: Some(depth_texture_view),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, viewport: &camera::Viewport) {
        self.depth_texture_view = Some(texture::create_view(
            device,
            viewport.width,
            viewport.height,
            settings::DEPTH_TEXTURE_FORMAT,
        ));

        match &mut self.mode {
            Mode::Smaa(smaa) => smaa.resize(device, viewport.width, viewport.height),
            Mode::Fxaa(fxaa) => fxaa.resize(device, viewport.width, viewport.height),
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

    pub fn toggle(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) {
        self.mode = match self.mode {
            Mode::Smaa(_) => Mode::Fxaa(create_fxaa(device, viewport)),
            Mode::Fxaa(_) => Mode::None,
            Mode::None => Mode::Smaa(create_smaa(device, queue, viewport)),
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

fn create_smaa(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> SmaaTarget {
    SmaaTarget::new(
        &device,
        &queue,
        viewport.width,
        viewport.height,
        settings::COLOR_TEXTURE_FORMAT,
        SmaaMode::Smaa1X,
    )
}

fn create_fxaa(device: &wgpu::Device, viewport: &camera::Viewport) -> pipelines::fxaa::Fxaa {
    pipelines::fxaa::Fxaa::new(&device, viewport.width, viewport.height)
}
