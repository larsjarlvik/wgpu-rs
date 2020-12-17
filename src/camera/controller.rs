use crate::KeyboardInput;
use std::time::Duration;
use winit::event::*;

pub struct CameraController {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_zoom_in_pressed: bool,
    is_zoom_out_pressed: bool,
    forward_speed: f32,
    strafe_speed: f32,
    zoom_speed: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_zoom_in_pressed: false,
            is_zoom_out_pressed: false,
            forward_speed: 0.0,
            strafe_speed: 0.0,
            zoom_speed: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                    }
                    VirtualKeyCode::A => {
                        self.is_left_pressed = is_pressed;
                    }
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                    }
                    VirtualKeyCode::D => {
                        self.is_right_pressed = is_pressed;
                    }
                    VirtualKeyCode::Up => {
                        self.is_zoom_in_pressed = is_pressed;
                    }
                    VirtualKeyCode::Down => {
                        self.is_zoom_out_pressed = is_pressed;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update_camera(&mut self, frame_time: &Duration) -> cgmath::Vector3<f32> {
        let max_speed = 0.5;
        let mut acceleration = 0.2 * frame_time.as_millis() as f32;

        if acceleration > max_speed {
            acceleration = max_speed
        };
        if acceleration < -max_speed {
            acceleration = -max_speed
        };

        if self.is_forward_pressed {
            self.forward_speed -= acceleration;
        }
        if self.is_left_pressed {
            self.strafe_speed -= acceleration;
        }
        if self.is_backward_pressed {
            self.forward_speed += acceleration;
        }
        if self.is_right_pressed {
            self.strafe_speed += acceleration;
        }

        if self.is_zoom_in_pressed {
            self.zoom_speed -= acceleration;
        }
        if self.is_zoom_out_pressed {
            self.zoom_speed += acceleration;
        }

        self.forward_speed *= 0.9;
        self.strafe_speed *= 0.9;
        self.zoom_speed *= 0.8;

        cgmath::Vector3::new(self.strafe_speed, self.zoom_speed, self.forward_speed)
    }
}
