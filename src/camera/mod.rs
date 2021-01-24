use crate::{input, settings};
use cgmath::*;
use winit::event::VirtualKeyCode;
use SquareMatrix;
mod controller;
pub mod frustum;
mod uniforms;

pub struct Camera {
    controller: controller::CameraController,
    pub uniforms: uniforms::UniformBuffer,
    pub target: Point3<f32>,
    pub eye: Point3<f32>,
    pub rotation: Point2<f32>,
    pub distance: f32,
    pub width: f32,
    pub height: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub z_far_range: f32,
    pub proj: Matrix4<f32>,
    pub frustum: frustum::FrustumCuller,
}

impl Camera {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let z_far_range = num_traits::Float::sqrt(z_far * z_far + z_far * z_far);
        let width = swap_chain_desc.width as f32;
        let height = swap_chain_desc.height as f32;
        let fov_y = 45.0;
        let controller = controller::CameraController::new();
        let frustum = frustum::FrustumCuller::new();

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            uniforms::Uniforms {
                view: Matrix4::identity().into(),
                proj: Matrix4::identity().into(),
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
            eye: Point3::new(0.0, 0.0, 0.0),
            rotation: Point2::new(45.0f32.to_radians(), -90.0f32.to_radians()),
            distance: 100.0,
            width: swap_chain_desc.width as f32,
            height: swap_chain_desc.height as f32,
            fov_y,
            z_near,
            z_far,
            z_far_range,
            proj: perspective(Deg(fov_y), width / height, z_near, z_far),
            frustum,
        }
    }

    pub fn resize(&mut self, swap_chain_desc: &wgpu::SwapChainDescriptor) {
        self.width = swap_chain_desc.width as f32;
        self.height = swap_chain_desc.height as f32;
        self.uniforms.data.viewport_size = [self.width, self.height];
        self.update_proj();
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, input: &input::Input, frame_time: f32) {
        if input.keys.contains(&VirtualKeyCode::PageUp) {
            self.z_far += 1.0;
            self.update_proj();
        }
        if input.keys.contains(&VirtualKeyCode::PageDown) && self.z_far > 400.0 {
            self.z_far -= 1.0;
            self.update_proj();
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

        let view = Matrix4::look_at(self.eye, self.target, Vector3::unit_y());

        self.frustum = frustum::FrustumCuller::from_matrix(self.proj * view);

        self.uniforms.data.look_at = self.target.into();
        self.uniforms.data.eye_pos = self.eye.into();
        self.uniforms.data.view = view.into();
        self.uniforms.data.proj = self.proj.into();
        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }

    pub fn get_lod(&self, target: Point3<f32>) -> u32 {
        let distance = self.eye.distance(target) / self.z_far;
        for i in 0..settings::LODS.len() {
            let lod = settings::LODS.get(i).expect("Failed to get LOD!");
            if lod > &distance {
                return i as u32;
            }
        }

        settings::LODS.len() as u32
    }

    fn update_proj(&mut self) {
        self.proj = perspective(Deg(self.fov_y), self.width / self.height, self.z_near, self.z_far);
    }
}
