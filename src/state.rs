use crate::camera;
use crate::camera_controller;
use crate::default_render_pipeline;
use crate::model;
use crate::texture;
use crate::vertex::*;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pipelines: Vec<default_render_pipeline::RenderPipeline>,
    camera: camera::Camera,
    camera_controller: camera_controller::CameraController,
    model: model::Model,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        // Device
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

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
            .unwrap();

        // Swap chain
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        // Texture
        let default_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("./textures/texture.png"),
            "happy-tree.png",
        )
        .unwrap();

        let brdf_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("./textures/brdf_lut.png"),
            "brdf_lut",
        )
        .unwrap();

        // Render pipelines
        let mut pipelines = Vec::new();
        let mut pipeline1 = default_render_pipeline::RenderPipeline::new(&device, &default_texture);
        pipeline1.uniforms.transform = cgmath::Matrix4::from_translation(cgmath::Vector3 { x:-0.5, y: 0.0, z: 0.0 }).into();
        pipelines.push(pipeline1);
        let mut pipeline2 = default_render_pipeline::RenderPipeline::new(&device, &brdf_texture);
        pipeline2.uniforms.transform = cgmath::Matrix4::from_translation(cgmath::Vector3 { x: 0.5, y: 0.0, z:-0.5 }).into();
        pipelines.push(pipeline2);

        let camera_controller = camera_controller::CameraController::new(0.2);
        let model = model::Model::new(&device);

        // Camera
        let camera = camera::Camera::new(swap_chain_desc.width as f32 / swap_chain_desc.height as f32);

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            pipelines,
            camera,
            camera_controller,
            model,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.camera = camera::Camera::new(self.swap_chain_desc.width as f32 / self.swap_chain_desc.height as f32);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);

        let view_proj = self.camera.build_view_projection_matrix().into();
        for i in 0..2 {
            self.pipelines[i].uniforms.view_proj = view_proj;
            self.pipelines[i].queue_uniforms(&self.queue);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            for pipeline in &self.pipelines {
                render_pass.set_pipeline(&pipeline.render_pipeline);
                render_pass.set_bind_group(0, &pipeline.texture, &[]);
                render_pass.set_bind_group(1, &pipeline.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.model.index_buffer.slice(..));
                render_pass.draw_indexed(0..self.model.num_indices, 0, 1..2);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
