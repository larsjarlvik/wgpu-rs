use crate::input;
use cgmath::*;
use winit::event::VirtualKeyCode;
mod controller;
pub mod frustum;
mod uniforms;

pub struct Instance {
    pub uniforms: uniforms::UniformBuffer,
    pub frustum: frustum::FrustumCuller,
}

pub struct Controller {
    controller: controller::CameraController,
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
}

impl Controller {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let fov_y = 45.0;
        let controller = controller::CameraController::new();

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

        Controller {
            controller,
            target: Point3::new(0.0, 0.0, 0.0),
            eye: Point3::new(0.0, 0.0, 0.0),
            rotation: Point2::new(45.0f32.to_radians(), -90.0f32.to_radians()),
            distance: 100.0,
            width: swap_chain_desc.width,
            height: swap_chain_desc.height,
            fov_y,
            z_near,
            z_far,
            proj: Matrix4::identity(),
            bind_group_layout,
        }
    }

    pub fn resize(&mut self, swap_chain_desc: &wgpu::SwapChainDescriptor) {
        self.width = swap_chain_desc.width;
        self.height = swap_chain_desc.height;
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
}

impl Instance {
    pub fn from_controller(device: &wgpu::Device, cameras: &Controller, clip: [f32; 4]) -> Self {
        let frustum = frustum::FrustumCuller::new();

        Self {
            uniforms: uniforms::UniformBuffer::new(
                &device,
                &cameras.bind_group_layout,
                uniforms::Uniforms {
                    view_proj: Matrix4::identity().into(),
                    eye_pos: [0.0, 0.0, 0.0],
                    look_at: [0.0, 0.0, 0.0],
                    z_near: cameras.z_near,
                    z_far: cameras.z_far,
                    viewport_size: [cameras.width as f32, cameras.height as f32],
                    clip,
                },
            ),
            frustum,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, target: Point3<f32>, eye: Point3<f32>, world_matrix: Matrix4<f32>) {
        self.frustum = frustum::FrustumCuller::from_matrix(world_matrix);
        self.uniforms.data.look_at = target.into();
        self.uniforms.data.eye_pos = eye.into();
        self.uniforms.data.view_proj = (world_matrix).into();

        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
