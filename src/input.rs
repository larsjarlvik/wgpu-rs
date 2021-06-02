use std::collections::{HashMap, HashSet};
use winit::{dpi::PhysicalPosition, event::*};

pub struct Input {
    pub keys: HashMap<VirtualKeyCode, bool>,
    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_delta: (f32, f32),
    pub mouse_scroll_delta: f32,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys: HashMap::new(),
            mouse_buttons: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_scroll_delta: 0.0,
        }
    }

    pub fn process_key(&mut self, input: &KeyboardInput) {
        if let Some(key_code) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                if !self.keys.contains_key(&key_code) {
                    self.keys.insert(key_code, false);
                }
            } else {
                self.keys.remove(&key_code);
            }
        }
    }

    pub fn process_mouse_button(&mut self, button: &MouseButton, state: &ElementState) {
        if *state == ElementState::Pressed {
            self.mouse_buttons.insert(*button);
        } else {
            self.mouse_buttons.remove(button);
        }
    }

    pub fn process_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta: (x, y), .. } => {
                self.mouse_delta = (*x as f32, *y as f32);
            }
            DeviceEvent::MouseWheel { delta, .. } => {
                self.mouse_scroll_delta = -match delta {
                    MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
                };
            }
            _ => {}
        }
    }

    pub fn after_update(&mut self) {
        self.mouse_scroll_delta = 0.0;
        self.mouse_delta = (0.0, 0.0);

        for (_, repeat) in self.keys.iter_mut() {
            *repeat = true;
        }
    }
}
