use crate::{input, settings};
use cgmath::*;
use winit::event::VirtualKeyCode;
mod bounding_box;
mod controller;
mod frustum;
mod instance;
mod uniforms;

pub use self::bounding_box::BoundingBox;
pub use self::frustum::FrustumCuller;
pub use self::instance::Instance;

pub struct Viewport {
    controller: controller::Controller,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub target: Point3<f32>,
    pub eye: Point3<f32>,
    pub rotation: Point2<f32>,
    pub distance: f32,
    pub width: u32,
    pub height: u32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub proj: Matrix4<f32>,
    pub valid: bool,
}

impl Viewport {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let fov_y = 45.0;
        let controller = controller::Controller::new();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        Viewport {
            controller,
            target: Point3::new(0.0, 0.0, 0.0),
            eye: Point3::new(0.0, 0.0, 0.0),
            rotation: Point2::new(45.0f32.to_radians(), -90.0f32.to_radians()),
            distance: 100.0,
            width,
            height,
            fov_y,
            z_near,
            z_far,
            proj: Matrix4::identity(),
            bind_group_layout,
            valid: true,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn update(&mut self, input: &input::Input, frame_time: f32) {
        if input.keys.contains(&VirtualKeyCode::PageUp) {
            self.z_far += 1.0;
        }
        if input.keys.contains(&VirtualKeyCode::PageDown) && self.z_far > 400.0 {
            self.z_far -= 1.0;
        }

        self.controller.process_events(input, frame_time);

        self.distance += self.controller.velocity.y;
        self.rotation += self.controller.rotation;
        self.target.x += self.controller.velocity.z * self.rotation.y.cos();
        self.target.x += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).cos();
        self.target.z += self.controller.velocity.z * self.rotation.y.sin();
        self.target.z += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).sin();

        self.eye = Point3::new(
            self.target.x - (self.rotation.x.sin() * self.rotation.y.cos() * self.distance),
            self.target.y + self.rotation.x.cos() * self.distance,
            self.target.z - (self.rotation.x.sin() * self.rotation.y.sin() * self.distance),
        );

        self.proj = perspective(Deg(self.fov_y), self.width as f32 / self.height as f32, self.z_near, self.z_far);
    }

    pub fn create_swap_chain(&self, device: &wgpu::Device, surface: &wgpu::Surface) -> wgpu::SwapChain {
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: settings::COLOR_TEXTURE_FORMAT,
            width: self.width,
            height: self.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        device.create_swap_chain(&surface, &swap_chain_desc)
    }
}
