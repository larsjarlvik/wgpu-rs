use crate::camera;
use crate::camera_controller;
use crate::drawable::Drawable;
use crate::pipeline::*;
use crate::vertex::*;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: render::RenderPipeline,
    drawables: Vec<Drawable>,
    camera: camera::Camera,
    camera_controller: camera_controller::CameraController,
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

        // Sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Drawing
        let render_pipeline = render::RenderPipeline::new(&device);
        let mut drawables = Vec::new();

        let t1 = texture::Texture::load_image("./textures/texture.png").unwrap();
        let mut d1 = Drawable::new(&device, &render_pipeline, &queue, &sampler, &t1);
        d1.uniforms.data.transform = cgmath::Matrix4::from_translation(cgmath::Vector3 { x: -0.5, y: 0.0, z: 0.0 }).into();
        drawables.push(d1);

        let t2 = texture::Texture::load_image("./textures/brdf_lut.png").unwrap();
        let mut d2 = Drawable::new(&device, &render_pipeline, &queue, &sampler, &t2);
        d2.uniforms.data.transform = cgmath::Matrix4::from_translation(cgmath::Vector3 { x: 0.5, y: 0.0, z: 0.0 }).into();
        drawables.push(d2);

        // Camera
        let camera_controller = camera_controller::CameraController::new(0.2);
        let camera = camera::Camera::new(swap_chain_desc.width as f32 / swap_chain_desc.height as f32);

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            render_pipeline,
            drawables,
            camera,
            camera_controller,
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
            self.drawables[i].uniforms.data.view_proj = view_proj;
            self.queue.write_buffer(
                &self.drawables[i].uniforms.buffer,
                0,
                bytemuck::cast_slice(&[self.drawables[i].uniforms.data]),
            );
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

            render_pass.set_pipeline(&self.render_pipeline.render_pipeline);
            for drawable in &self.drawables {
                render_pass.set_bind_group(0, &drawable.texture_bind_group, &[]);
                render_pass.set_bind_group(1, &drawable.uniforms.bind_group, &[]);
                render_pass.set_vertex_buffer(0, drawable.model.vertex_buffer.slice(..));
                render_pass.set_index_buffer(drawable.model.index_buffer.slice(..));
                render_pass.draw_indexed(0..drawable.model.num_indices, 0, 1..2);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
