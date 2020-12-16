use crate::camera_controller;
use crate::models;
use crate::settings;
use crate::{camera, deferred};
use rand::prelude::*;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    camera: camera::Camera,
    camera_controller: camera_controller::CameraController,
    models: models::Models,
    deferred_render: deferred::DeferredRender,
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
        let deferred_render = deferred::DeferredRender::new(&device, &swap_chain_desc);

        // Camera
        let camera_controller = camera_controller::CameraController::new(0.2);
        let camera = camera::Camera::new(swap_chain_desc.width as f32 / swap_chain_desc.height as f32);

        // Drawing
        let mut rng = rand::thread_rng();
        let mut models = models::Models::new(&device);
        models.load_model(&device, &queue, "pine", "pine.glb");
        for _ in 0..10000 {
            models.add_instance(
                "pine",
                models::data::Instance {
                    transform: {
                        cgmath::Matrix4::from_translation(cgmath::Vector3 {
                            x: (rng.gen::<f32>() - 0.5) * 500.0,
                            y: 0.0,
                            z: (rng.gen::<f32>() - 0.5) * 500.0,
                        }) * cgmath::Matrix4::from_scale(rng.gen::<f32>() * 2.0 + 0.5)
                    }
                    .into(),
                },
            );
        }
        models.write_instance_buffers(&device, "pine");
        models.refresh_render_bundle(&device);

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            deferred_render,
            camera,
            camera_controller,
            models,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.camera = camera::Camera::new(self.swap_chain_desc.width as f32 / self.swap_chain_desc.height as f32);
        self.deferred_render = deferred::DeferredRender::new(&self.device, &self.swap_chain_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        let view_proj = self.camera.build_view_projection_matrix().into();
        self.models.set_uniforms(&self.queue, models::data::Uniforms { view_proj });
        self.deferred_render.set_camera(&self.queue, self.camera.eye);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        // Main render pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
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
                .execute_bundles(std::iter::once(&self.models.render_bundle));

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
