use std::collections::HashSet;
use winit::event::*;

pub struct Input {
    pub keys: HashSet<VirtualKeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Input { keys: HashSet::new() }
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
                if *state == ElementState::Pressed {
                    self.keys.insert(*keycode);
                } else {
                    self.keys.remove(keycode);
                }
            }
            _ => {}
        }
    }
}
