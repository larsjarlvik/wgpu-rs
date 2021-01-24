use std::time::Instant;

use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    window::Fullscreen,
};
mod camera;
mod deferred;
mod fxaa;
mod input;
mod models;
mod noise;
mod settings;
mod state;
mod texture;
mod world;
mod plane;
pub use state::*;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU-RS")
        .build(&event_loop)
        .expect("Failed to create window!");
    let mut state = block_on(state::State::new(&window));
    let mut fps = 0;
    let mut last_update = Instant::now();
    let mut is_focused = true;

    let monitor = event_loop.available_monitors().nth(0);
    let fullscreen = Some(Fullscreen::Borderless(monitor));

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { ref event, .. } => {
            if is_focused {
                state.input(event);
            }
        }
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("Render error: {:?}", e),
            }
            fps += 1;
            if last_update.elapsed().as_millis() > 1000 {
                window.set_title(format!("WGPU-RS: {} FPS", fps).as_str());
                last_update = Instant::now();
                fps = 0;
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::F),
                    ..
                } => {
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                    } else {
                        window.set_fullscreen(fullscreen.clone());
                    }
                 },
                _ => {}
            },
            WindowEvent::Focused(focused) => {
                is_focused = *focused;
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        },
        _ => {}
    });
}
