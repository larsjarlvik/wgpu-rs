use cgmath::*;
use super::{frustum, uniforms};

pub struct Camera {
    pub uniforms: uniforms::UniformBuffer,
    pub frustum: frustum::FrustumCuller,
    pub z_far_range: f32,
}

impl Camera {
    pub fn new(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, width: f32, height: f32, z_near: f32, z_far: f32, clip: [f32; 4]) -> Self {
        let z_far_range = num_traits::Float::sqrt(z_far * z_far + z_far * z_far);
        let frustum = frustum::FrustumCuller::new();

        Self {
            uniforms: uniforms::UniformBuffer::new(
                &device,
                &bind_group_layout,
                uniforms::Uniforms {
                    view_proj: Matrix4::identity().into(),
                    eye_pos: [0.0, 0.0, 0.0],
                    look_at: [0.0, 0.0, 0.0],
                    z_near,
                    z_far,
                    viewport_size: [width, height],
                    clip,
                },
            ),
            frustum,
            z_far_range,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, target: Point3<f32>, eye: Point3<f32>, world_matrix: Matrix4<f32>) {
        self.frustum = frustum::FrustumCuller::from_matrix(world_matrix);
        self.uniforms.data.look_at = target.into();
        self.uniforms.data.eye_pos = eye.into();
        self.uniforms.data.view_proj = (world_matrix).into();
        queue.write_buffer(
            &self.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms.data]),
        );

    }
}