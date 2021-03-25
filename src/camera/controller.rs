use crate::{input, settings};
use cgmath::*;
use winit::event::*;

pub struct Controller {
    pub velocity: Vector3<f32>,
    pub rotation: Vector2<f32>,
    acceleration: Vector3<f32>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            velocity: vec3(0.0, 0.0, 0.0),
            rotation: vec2(0.0, 0.0),
            acceleration: vec3(0.0, 0.0, 0.0),
        }
    }

    pub fn process_events(&mut self, input: &input::Input, frame_time: f32) {
        let time_step = frame_time * 0.01;
        self.acceleration = vec3(0.0, 0.0, 0.0);

        // Movement
        if input.keys.contains(&VirtualKeyCode::W) {
            self.acceleration.z += settings::CAMERA_ACCELERATION * time_step;
        }
        if input.keys.contains(&VirtualKeyCode::A) {
            self.acceleration.x += settings::CAMERA_ACCELERATION * time_step;
        }
        if input.keys.contains(&VirtualKeyCode::S) {
            self.acceleration.z -= settings::CAMERA_ACCELERATION * time_step;
        }
        if input.keys.contains(&VirtualKeyCode::D) {
            self.acceleration.x -= settings::CAMERA_ACCELERATION * time_step;
        }

        // Zoom
        if input.mouse_scroll_delta < 0.0 {
            self.acceleration.y -= settings::CAMERA_ACCELERATION;
        }
        if input.mouse_scroll_delta > 0.0 {
            self.acceleration.y += settings::CAMERA_ACCELERATION;
        }

        // Rotation
        if input.mouse_buttons.contains(&MouseButton::Right) {
            self.update_rotation(input.mouse_delta, time_step);
        } else {
            self.update_rotation((0.0, 0.0), time_step);
        }

        self.update_position(time_step);
    }

    fn update_position(&mut self, time_step: f32) {
        self.velocity += vec3(
            self.acceleration.x - settings::CAMERA_FRICTION * self.velocity.x,
            self.acceleration.y - settings::CAMERA_FRICTION * self.velocity.y,
            self.acceleration.z - settings::CAMERA_FRICTION * self.velocity.z,
        ) * time_step;
    }

    fn update_rotation(&mut self, mouse_delta: (f32, f32), time_step: f32) {
        let (x, y) = mouse_delta;

        self.rotation = vec2(
            y.to_radians() * time_step * settings::CAMERA_SENSITIVITY,
            -x.to_radians() * time_step * settings::CAMERA_SENSITIVITY,
        );
    }
}
