use crate::input;
use cgmath::*;
use winit::event::VirtualKeyCode;
pub mod camera;
mod controller;
pub mod frustum;
mod uniforms;

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
    pub eye_cam: camera::Camera,
    pub refraction_cam: camera::Camera,
    pub reflection_cam: camera::Camera,
}

impl Cameras {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let z_near = 1.0;
        let z_far = 800.0;
        let width = swap_chain_desc.width as f32;
        let height = swap_chain_desc.height as f32;
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

        let eye_cam = camera::Camera::new(device, &bind_group_layout, width, height, z_near, z_far, [0.0, 1.0, 0.0, 1.0], 1.0);
        let reflection_cam = camera::Camera::new(device, &bind_group_layout, width, height, z_near, z_far, [0.0, 1.0, 0.0, 1.0], -1.0);
        let refraction_cam = camera::Camera::new(device, &bind_group_layout, width, height, z_near, z_far, [0.0, -1.0, 0.0, 1.0], 1.0);

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

        self.eye_cam.update(queue, self.target, eye, world_matrix);
        self.refraction_cam.update(queue, self.target, eye, world_matrix);

        let view = Matrix4::look_at(Point3::new(eye.x, -eye.y, eye.z), self.target, -Vector3::unit_y());
        self.reflection_cam.update(queue, self.target, eye, proj * view);
    }
}
