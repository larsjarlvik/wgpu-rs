use crate::{input, settings};
use cgmath::*;
use SquareMatrix;
mod controller;
mod uniforms;

pub struct Camera {
    controller: controller::CameraController,
    pub uniforms: uniforms::UniformBuffer,
    pub target: Point3<f32>,
    pub rotation: Point2<f32>,
    pub distance: f32,
    pub width: f32,
    pub height: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub proj: Matrix4<f32>,
}

impl Camera {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let width = swap_chain_desc.width as f32;
        let height = swap_chain_desc.height as f32;
        let fov_y = 45.0;
        let controller = controller::CameraController::new();

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            uniforms::Uniforms {
                view_proj: Matrix4::identity().into(),
                eye_pos: [0.0, 0.0, 0.0],
                look_at: [0.0, 0.0, 0.0],
                z_near,
                z_far,
                viewport_size: [width, height],
            },
        );
        Camera {
            controller,
            uniforms,
            target: Point3::new(0.0, 0.0, 0.0),
            rotation: Point2::new(45.0f32.to_radians(), 90.0f32.to_radians()),
            distance: 100.0,
            width: swap_chain_desc.width as f32,
            height: swap_chain_desc.height as f32,
            fov_y,
            z_near,
            z_far,
            proj: perspective(Deg(fov_y), width / height, z_near, z_far),
        }
    }

    pub fn resize(&mut self, swap_chain_desc: &wgpu::SwapChainDescriptor) {
        self.width = swap_chain_desc.width as f32;
        self.height = swap_chain_desc.height as f32;
        self.uniforms.data.viewport_size = [self.width, self.height];
        self.proj = perspective(Deg(self.fov_y), self.width / self.height, self.z_near, self.z_far);
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, frame_time: f32, input: &input::Input) {
        self.controller.process_events(input, frame_time);

        self.distance += self.controller.velocity.y;
        self.rotation += self.controller.rotation;
        self.target.x += self.controller.velocity.z * self.rotation.y.cos();
        self.target.x += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).cos();
        self.target.z += self.controller.velocity.z * self.rotation.y.sin();
        self.target.z += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).sin();

        let eye = Point3::new(
            self.target.x - (self.rotation.x.sin() * self.rotation.y.cos() * self.distance),
            self.target.y + self.rotation.x.cos() * self.distance,
            self.target.z - (self.rotation.x.sin() * self.rotation.y.sin() * self.distance),
        );

        let view = Matrix4::look_at(eye, self.target, Vector3::unit_y());
        self.uniforms.data.look_at = self.target.into();
        self.uniforms.data.eye_pos = eye.into();
        self.uniforms.data.view_proj = (settings::OPENGL_TO_WGPU_MATRIX * self.proj * view).into();

        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
