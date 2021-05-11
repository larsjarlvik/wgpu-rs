use crate::{camera, pipelines, settings, texture};
use smaa::{SmaaMode, SmaaTarget};

pub enum Mode {
    Smaa,
    Fxaa,
    None,
}

pub struct AntiAliasing {
    pub mode: Mode,
    pub depth_texture_view: wgpu::TextureView,
    fxaa: pipelines::fxaa::Fxaa,
    smaa: SmaaTarget,
}

impl AntiAliasing {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Self {
        let depth_texture_view = texture::create_view(device, viewport.width, viewport.height, settings::DEPTH_TEXTURE_FORMAT);
        let fxaa = pipelines::fxaa::Fxaa::new(&device, viewport.width, viewport.height);
        let smaa = SmaaTarget::new(
            &device,
            &queue,
            viewport.width,
            viewport.height,
            settings::COLOR_TEXTURE_FORMAT,
            SmaaMode::Smaa1X,
        );

        Self {
            mode: Mode::Smaa,
            fxaa,
            smaa,
            depth_texture_view,
        }
    }

    pub fn execute<F: Fn(&wgpu::TextureView, &wgpu::TextureView)>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        f: F,
    ) {
        match self.mode {
            Mode::Fxaa => {
                f(&self.fxaa.texture_view, &self.depth_texture_view);
            }
            Mode::Smaa => {
                f(&self.smaa.start_frame(device, queue, view), &self.depth_texture_view);
            }
            Mode::None => {
                f(&view, &self.depth_texture_view);
            }
        };

        match self.mode {
            Mode::Fxaa => {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
                self.fxaa.render(&mut encoder, &view);
                queue.submit(std::iter::once(encoder.finish()));
            }
            _ => {}
        }
    }

    pub fn toggle(&mut self) {
        self.mode = match self.mode {
            Mode::Smaa => Mode::Fxaa,
            Mode::Fxaa => Mode::None,
            Mode::None => Mode::Smaa,
        }
    }

    pub fn display(&self) -> &str {
        match self.mode {
            Mode::Fxaa => "FXAA",
            Mode::Smaa => "SMAA",
            Mode::None => "None",
        }
    }
}
