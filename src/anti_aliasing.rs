use crate::{camera, pipelines, settings};
use smaa::{SmaaMode, SmaaTarget};

pub enum Mode {
    Smaa,
    Fxaa,
    None,
}

pub struct AntiAliasing {
    pub mode: Mode,
    fxaa: pipelines::fxaa::Fxaa,
    smaa: SmaaTarget,
}

impl AntiAliasing {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Self {
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
        }
    }

    pub fn execute<F: Fn(&wgpu::TextureView)>(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, f: F) {
        match self.mode {
            Mode::Fxaa => {
                f(&self.fxaa.texture_view);
            }
            Mode::Smaa => {
                f(&self.smaa.start_frame(device, queue, view));
            }
            Mode::None => {
                f(&view);
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
