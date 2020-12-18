use cgmath::SquareMatrix;
use std::time::Duration;

use crate::input;

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
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let controller = controller::CameraController::new();
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
            target: (0.0, 0.0, 0.0).into(),
            rotation: ((45.0 as f32).to_radians(), 0.0).into(),
            distance: 100.0,
            aspect: aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 500.0,
        }
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, frame_time: &Duration, input: &input::Input) {
        self.controller.process_events(input, frame_time);

        let move_velocity = self.controller.velocity;
        self.target += cgmath::Vector3::new(move_velocity.x, 0.0, move_velocity.z);
        self.distance += move_velocity.y;
        self.rotation += self.controller.rotation;

        let eye = cgmath::Point3::new(
            self.target.x - (self.rotation.x.sin() * self.rotation.y.cos() * self.distance),
            self.rotation.x.cos() * self.distance,
            self.target.z - (self.rotation.x.sin() * self.rotation.y.sin() * self.distance),
        );

        let view = cgmath::Matrix4::look_at(eye, self.target, cgmath::Vector3::unit_y());
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        self.uniforms.data.eye_pos = eye.into();
        self.uniforms.data.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();

        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
