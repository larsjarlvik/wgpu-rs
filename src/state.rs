use crate::{camera, deferred, input::Input, models, settings, world};
use std::time::Instant;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    camera: camera::Camera,
    models: models::Models,
    world: world::World,
    deferred_render: deferred::DeferredRender,
    last_frame: Instant,
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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
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
        let camera = camera::Camera::new(&device, swap_chain_desc.width as f32 / swap_chain_desc.height as f32);

        // Drawing
        let deferred_render = deferred::DeferredRender::new(&device, &swap_chain_desc, &camera);
        let mut models = models::Models::new(&device, &camera);

        // World
        let world = world::World::new(&device, &queue, &camera, &mut models);

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            deferred_render,
            camera,
            models,
            world,
            input: Input::new(),
            last_frame: Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.camera.aspect = self.swap_chain_desc.width as f32 / self.swap_chain_desc.height as f32;
        self.deferred_render = deferred::DeferredRender::new(&self.device, &self.swap_chain_desc, &self.camera);
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        &self.input.process_events(event);
    }

    pub fn update(&mut self) {
        let frame_time = self.last_frame.elapsed();
        self.last_frame = Instant::now();
        self.camera.update_camera(&self.queue, &frame_time, &self.input);
        self.world.update(&self.device, &self.camera, &mut self.models);
        self.input.after_update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // Main render pass
            let mut bundles = vec![];
            bundles.push(&self.world.terrain.render_bundle);
            bundles.push(&self.models.render_bundle);

            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.deferred_render.position_texture_view,
                            resolve_target: None,
                            ops,
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.deferred_render.normals_texture_view,
                            resolve_target: None,
                            ops,
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.deferred_render.base_color_texture_view,
                            resolve_target: None,
                            ops,
                        },
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &self.deferred_render.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                })
                .execute_bundles(bundles.into_iter());

            // Deferred render pass
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops,
                    }],
                    depth_stencil_attachment: None,
                })
                .execute_bundles(std::iter::once(&self.deferred_render.render_bundle));
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
