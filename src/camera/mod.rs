use crate::input;
use cgmath::SquareMatrix;
mod controller;
mod uniforms;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    controller: controller::CameraController,
    pub uniforms: uniforms::UniformBuffer,
    pub target: cgmath::Point3<f32>,
    pub rotation: cgmath::Point2<f32>,
    pub distance: f32,
    pub width: f32,
    pub height: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 400.0;
        let controller = controller::CameraController::new();

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            uniforms::Uniforms {
                view_proj: cgmath::Matrix4::identity().into(),
                eye_pos: [0.0, 0.0, 0.0],
                look_at: [0.0, 0.0, 0.0],
                z_near,
                z_far,
                viewport_size: [swap_chain_desc.width as f32, swap_chain_desc.height as f32],
            },
        );
        Camera {
            controller,
            uniforms,
            target: (0.0, 0.0, 0.0).into(),
            rotation: ((45.0 as f32).to_radians(), (90.0 as f32).to_radians()).into(),
            distance: 100.0,
            width: swap_chain_desc.width as f32,
            height: swap_chain_desc.height as f32,
            fov_y: 45.0,
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, swap_chain_desc: &wgpu::SwapChainDescriptor) {
        self.width = swap_chain_desc.width as f32;
        self.height = swap_chain_desc.height as f32;
        self.uniforms.data.viewport_size = [self.width, self.height];
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, frame_time: f32, input: &input::Input) {
        self.controller.process_events(input, frame_time);

        self.distance += self.controller.velocity.y;
        self.rotation += self.controller.rotation;
        self.target.x += self.controller.velocity.z * self.rotation.y.cos();
        self.target.x += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).cos();
        self.target.z += self.controller.velocity.z * self.rotation.y.sin();
        self.target.z += self.controller.velocity.x * (self.rotation.y - std::f32::consts::PI / 2.0).sin();

        let eye = cgmath::Point3::new(
            self.target.x - (self.rotation.x.sin() * self.rotation.y.cos() * self.distance),
            self.target.y + self.rotation.x.cos() * self.distance,
            self.target.z - (self.rotation.x.sin() * self.rotation.y.sin() * self.distance),
        );

        let view = cgmath::Matrix4::look_at(eye, self.target, cgmath::Vector3::unit_y());
        let proj = cgmath::perspective(cgmath::Deg(self.fov_y), self.width / self.height, self.z_near, self.z_far);

        self.uniforms.data.look_at = self.target.into();
        self.uniforms.data.eye_pos = eye.into();
        self.uniforms.data.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();

        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
