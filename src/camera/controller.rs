use cgmath::{Vector2, Vector3};
use std::time::Duration;
use winit::event::*;

use crate::{input, settings};

pub struct CameraController {
    pub velocity: Vector3<f32>,
    pub rotation: Vector2<f32>,
    acceleration: Vector3<f32>,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector2::new(0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn process_events(&mut self, input: &input::Input, frame_time: &Duration) {
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
        if input.mouse_scroll_delta < 0.0 {
            self.acceleration.y -= settings::CAMERA_ACCELERATION;
        }
        if input.mouse_scroll_delta > 0.0 {
            self.acceleration.y += settings::CAMERA_ACCELERATION;
        }

        if input.mouse_buttons.contains(&3) {
            self.update_rotation(input.mouse_delta);
        } else {
            self.update_rotation((0.0, 0.0));
        }

        self.update_position(frame_time);
    }

    fn update_position(&mut self, frame_time: &Duration) {
        let time_step = frame_time.as_millis() as f32 * 0.01;

        let temp_accel = Vector3::new(
            self.acceleration.x - settings::CAMERA_FRICTION * self.velocity.x * time_step,
            self.acceleration.y - settings::CAMERA_FRICTION * self.velocity.y * time_step,
            self.acceleration.z - settings::CAMERA_FRICTION * self.velocity.z * time_step,
        );

        self.velocity += temp_accel * time_step;
    }

    fn update_rotation(&mut self, mouse_delta: (f32, f32)) {
        let (x, y) = mouse_delta;
        self.rotation = Vector2::new(-y.to_radians(), x.to_radians());
    }
}
