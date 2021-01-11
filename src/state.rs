use crate::{camera, deferred, fxaa, input::Input, settings, world};
use std::time::Instant;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    camera: camera::Camera,
    world: world::World,
    deferred_render: deferred::DeferredRender,
    fxaa: fxaa::Fxaa,
    last_frame: Instant,
    frame_time: Vec<f32>,
    input: Input,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        // Device
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                    limits: wgpu::Limits {
                        max_bind_groups: 8,
                        ..wgpu::Limits::default()
                    },
                    shader_validation: true,
                },
                None,
            )
            .await
            .expect("Failed to request device");

        // Swap chain
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: settings::COLOR_TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        // Camera
        let camera = camera::Camera::new(&device, &swap_chain_desc);

        // Drawing
        let deferred_render = deferred::DeferredRender::new(&device, &queue, &swap_chain_desc, &camera);
        let fxaa = fxaa::Fxaa::new(&device, &swap_chain_desc);

        // World
        let world = world::World::new(&device, &queue, &camera).await;

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            deferred_render,
            fxaa,
            camera,
            world,
            input: Input::new(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.camera.resize(&self.swap_chain_desc);
        self.deferred_render = deferred::DeferredRender::new(&self.device, &self.queue, &self.swap_chain_desc, &self.camera);
        self.fxaa = fxaa::Fxaa::new(&self.device, &self.swap_chain_desc);
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        &self.input.process_events(event);
    }

    fn frame_time(&mut self) -> f32 {
        let avg_count = 30;
        self.frame_time.push(self.last_frame.elapsed().as_micros() as f32 / 1000.0);
        self.last_frame = Instant::now();

        if self.frame_time.len() > avg_count {
            self.frame_time.drain((self.frame_time.len() - avg_count)..);
        }
        self.frame_time.iter().sum::<f32>() / self.frame_time.len() as f32
    }

    pub fn update(&mut self) {
        let avg = self.frame_time();
        self.camera.update_camera(&self.queue, &self.input, avg);
        self.input.after_update();
        self.world.update(&self.device, &self.queue, &self.camera);
        self.deferred_render.update(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // World
            self.world.render(&mut encoder, ops, &self.deferred_render.textures);

            // Deferred
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.fxaa.texture_view,
                        resolve_target: None,
                        ops,
                    }],
                    depth_stencil_attachment: None,
                })
                .execute_bundles(std::iter::once(&self.deferred_render.render_bundle));

            // FXAA
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops,
                    }],
                    depth_stencil_attachment: None,
                })
                .execute_bundles(std::iter::once(&self.fxaa.render_bundle));
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
