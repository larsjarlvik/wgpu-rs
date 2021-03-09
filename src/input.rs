use std::collections::HashSet;
use winit::{dpi::PhysicalPosition, event::*};

pub struct Input {
    pub keys: HashSet<VirtualKeyCode>,
    pub mouse_buttons: HashSet<u32>,
    pub mouse_delta: (f32, f32),
    pub mouse_scroll_delta: f32,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_scroll_delta: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Key(KeyboardInput {
                state,
                virtual_keycode: Some(keycode),
                ..
            }) => {
                if *state == ElementState::Pressed {
                    self.keys.insert(*keycode);
                } else {
                    self.keys.remove(keycode);
                }
            }
            DeviceEvent::Button { state, button, .. } => {
                if *state == ElementState::Pressed {
                    self.mouse_buttons.insert(*button);
                } else {
                    self.mouse_buttons.remove(button);
                }
            }
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
    }
}
