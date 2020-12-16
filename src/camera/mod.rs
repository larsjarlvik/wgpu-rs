use cgmath::SquareMatrix;
use winit::event::WindowEvent;

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
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let controller = controller::CameraController::new(0.2);
        let uniforms = uniforms::UniformBuffer::new(
            &device,
            uniforms::Uniforms {
                view_proj: cgmath::Matrix4::identity().into(),
                eye_pos: [0.0, 0.0, 0.0],
            },
        );
        Camera {
            controller,
            uniforms,
            eye: (0.0, 50.0, 50.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 500.0,
        }
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue) {
        self.eye = self.controller.update_camera(&self);

        let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        self.uniforms.data.eye_pos = self.eye.into();
        self.uniforms.data.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();

        queue.write_buffer(
            &self.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms.data]),
        );
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        self.controller.process_events(&event)
    }
}
