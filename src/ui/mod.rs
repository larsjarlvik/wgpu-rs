use crate::settings;
use conrod_core::event::Input;
use conrod_wgpu::Image;
use std::fs;
mod config;
pub mod screens;

pub struct Ui {
    pub ui: conrod_core::Ui,
    pub load_op: wgpu::LoadOp<wgpu::Color>,
    renderer: conrod_wgpu::Renderer,
    image_map: conrod_core::image::Map<Image>,
    viewport: [f32; 4],
}

impl Ui {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let renderer = conrod_wgpu::Renderer::new(&device, 1, settings::COLOR_TEXTURE_FORMAT);
        let image_map = conrod_core::image::Map::new();
        let mut ui = conrod_core::UiBuilder::new([width as f64, height as f64]).build();
        let paths = fs::read_dir("./res/fonts").expect("Could not find ./res/fonts!");

        for font_path in paths {
            ui.fonts.insert_from_file(&font_path.unwrap().path()).unwrap();
        }

        Self {
            ui,
            renderer,
            image_map,
            viewport: [0.0, 0.0, width as f32, height as f32],
            load_op: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = [0.0, 0.0, width as f32, height as f32];
        self.ui.updated_widgets();
        if let Some(win_rect) = self.ui.rect_of(self.ui.window) {
            let (win_w, win_h) = (win_rect.w() as u32, win_rect.h() as u32);
            if width != win_w || height != win_h {
                let event = Input::Resize(width as f64, height as f64);
                self.ui.handle_event(event);
            }
        }
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::TextureView) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ui") });
        let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
            attachment: target,
            resolve_target: None,
            ops: wgpu::Operations {
                load: self.load_op,
                store: true,
            },
        };

        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("ui_render_pass"),
            color_attachments: &[color_attachment_desc],
            depth_stencil_attachment: None,
        };

        let primitives = self.ui.draw();
        if let Some(cmd) = self.renderer.fill(&self.image_map, self.viewport, 1.0, primitives).unwrap() {
            cmd.load_buffer_and_encode(&device, &mut encoder);
        }

        let render = self.renderer.render(&device, &self.image_map);
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_vertex_buffer(0, render.vertex_buffer.slice(..));

            for cmd in render.commands {
                match cmd {
                    conrod_wgpu::RenderPassCommand::SetPipeline { pipeline } => {
                        render_pass.set_pipeline(pipeline);
                    }
                    conrod_wgpu::RenderPassCommand::SetBindGroup { bind_group } => {
                        render_pass.set_bind_group(0, bind_group, &[]);
                    }
                    conrod_wgpu::RenderPassCommand::SetScissor { top_left, dimensions } => {
                        let [x, y] = top_left;
                        let [w, h] = dimensions;
                        render_pass.set_scissor_rect(x, y, w, h);
                    }
                    conrod_wgpu::RenderPassCommand::Draw { vertex_range } => {
                        render_pass.draw(vertex_range, 0..1);
                    }
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
