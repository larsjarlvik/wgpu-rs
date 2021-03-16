use super::{frustum, uniforms, Viewport};
use cgmath::*;

pub struct Instance {
    pub uniforms: uniforms::UniformBuffer,
    pub frustum: frustum::FrustumCuller,
}

impl Instance {
    pub fn from_controller(device: &wgpu::Device, cameras: &Viewport, clip: [f32; 4]) -> Self {
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.uniforms.data.viewport_size = [width as f32, height as f32];
    }
}
