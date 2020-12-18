use cgmath::Vector3;
use std::time::Duration;
use winit::event::*;

use crate::{input, settings};

pub struct CameraController {
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn process_events(&mut self, input: &input::Input) {
        self.acceleration = Vector3::new(0.0, 0.0, 0.0);
        if input.keys.contains(&VirtualKeyCode::W) {
            self.acceleration.z += settings::CAMERA_ACCELERATION;
        }
        if input.keys.contains(&VirtualKeyCode::A) {
            self.acceleration.x += settings::CAMERA_ACCELERATION;
        }
        if input.keys.contains(&VirtualKeyCode::S) {
            self.acceleration.z -= settings::CAMERA_ACCELERATION;
        }
        if input.keys.contains(&VirtualKeyCode::D) {
            self.acceleration.x -= settings::CAMERA_ACCELERATION;
        }
        if input.keys.contains(&VirtualKeyCode::Up) {
            self.acceleration.y -= settings::CAMERA_ACCELERATION;
        }
        if input.keys.contains(&VirtualKeyCode::Down) {
            self.acceleration.y += settings::CAMERA_ACCELERATION;
        }
    }

    pub fn update_camera(&mut self, frame_time: &Duration) -> Vector3<f32> {
        let time_step = frame_time.as_millis() as f32 * 0.01;

        let temp_accel = Vector3::new(
            self.acceleration.x - settings::CAMERA_FRICTION * self.velocity.x * time_step,
            self.acceleration.y - settings::CAMERA_FRICTION * self.velocity.y * time_step,
            self.acceleration.z - settings::CAMERA_FRICTION * self.velocity.z * time_step,
        );

        self.velocity += temp_accel * time_step;
        self.velocity
    }
}
