use crate::input;
use cgmath::*;
use winit::event::VirtualKeyCode;
use SquareMatrix;
mod controller;
pub mod frustum;
mod uniforms;

pub struct Camera {
    pub uniforms: uniforms::UniformBuffer,
    pub frustum: frustum::FrustumCuller,
    pub z_far_range: f32,
}

pub struct Cameras {
    controller: controller::CameraController,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub target: Point3<f32>,
    pub rotation: Point2<f32>,
    pub distance: f32,
    pub width: f32,
    pub height: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub eye_cam: Camera,
    pub refraction_cam: Camera,
    pub reflection_cam: Camera,
}

impl Cameras {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let z_far_range = num_traits::Float::sqrt(z_far * z_far + z_far * z_far);
        let width = swap_chain_desc.width as f32;
        let height = swap_chain_desc.height as f32;
        let fov_y = 45.0;
        let controller = controller::CameraController::new();
        let frustum = frustum::FrustumCuller::new();

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

        let eye_cam = Camera {
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
                    clip: [0.0, 0.0, 0.0, 0.0],
                },
            ),
            frustum,
            z_far_range,
        };
        let refraction_cam = Camera {
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
                    clip: [0.0, 0.0, 0.0, 0.0],
                },
            ),
            frustum,
            z_far_range,
        };
        let reflection_cam = Camera {
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
                    clip: [0.0, 0.0, 0.0, 0.0],
                },
            ),
            frustum,
            z_far_range,
        };

        Cameras {
            controller,
            target: Point3::new(0.0, 0.0, 0.0),
            rotation: Point2::new(45.0f32.to_radians(), -90.0f32.to_radians()),
            distance: 100.0,
            width: swap_chain_desc.width as f32,
            height: swap_chain_desc.height as f32,
            fov_y,
            z_near,
            z_far,
            eye_cam,
            refraction_cam,
            reflection_cam,
            bind_group_layout,
        }
    }

    pub fn resize(&mut self, swap_chain_desc: &wgpu::SwapChainDescriptor) {
        self.width = swap_chain_desc.width as f32;
        self.height = swap_chain_desc.height as f32;
        self.eye_cam.uniforms.data.viewport_size = vec2(self.width, self.height).into();
        self.refraction_cam.uniforms.data.viewport_size = vec2(self.width, self.height).into();
        self.reflection_cam.uniforms.data.viewport_size = vec2(self.width, self.height).into();
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, input: &input::Input, frame_time: f32) {
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

        let eye = Point3::new(
            self.target.x - (self.rotation.x.sin() * self.rotation.y.cos() * self.distance),
            self.target.y + self.rotation.x.cos() * self.distance,
            self.target.z - (self.rotation.x.sin() * self.rotation.y.sin() * self.distance),
        );

        let proj = perspective(Deg(self.fov_y), self.width / self.height, self.z_near, self.z_far);
        let view = Matrix4::look_at(eye, self.target, Vector3::unit_y());
        let world_matrix = proj * view;

        self.eye_cam.frustum = frustum::FrustumCuller::from_matrix(world_matrix);
        self.eye_cam.uniforms.data.look_at = self.target.into();
        self.eye_cam.uniforms.data.eye_pos = eye.into();
        self.eye_cam.uniforms.data.view_proj = (world_matrix).into();
        self.eye_cam.uniforms.data.clip = vec4(0.0, 1.0, 0.0, 1.0).into();
        queue.write_buffer(
            &self.eye_cam.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.eye_cam.uniforms.data]),
        );

        self.refraction_cam.frustum = frustum::FrustumCuller::from_matrix(world_matrix);
        self.refraction_cam.uniforms.data.look_at = self.target.into();
        self.refraction_cam.uniforms.data.eye_pos = eye.into();
        self.refraction_cam.uniforms.data.view_proj = (world_matrix).into();
        self.refraction_cam.uniforms.data.clip = vec4(0.0, -1.0, 0.0, 1.0).into();
        queue.write_buffer(
            &self.refraction_cam.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.refraction_cam.uniforms.data]),
        );

        let view = Matrix4::look_at(Point3::new(eye.x, -eye.y, eye.z), self.target, -Vector3::unit_y());
        let world_matrix = proj * view;

        self.reflection_cam.frustum = frustum::FrustumCuller::from_matrix(world_matrix);
        self.reflection_cam.uniforms.data.look_at = self.target.into();
        self.reflection_cam.uniforms.data.eye_pos = eye.into();
        self.reflection_cam.uniforms.data.view_proj = (world_matrix).into();
        self.reflection_cam.uniforms.data.clip = vec4(0.0, 1.0, 0.0, 1.0).into();
        queue.write_buffer(
            &self.reflection_cam.uniforms.buffer,
            0,
            bytemuck::cast_slice(&[self.reflection_cam.uniforms.data]),
        );
    }
}
