use futures::executor::block_on;
use std::time::Instant;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Fullscreen,
    window::WindowBuilder,
};
mod camera;
mod input;
mod logger;
mod models;
mod noise;
mod pipelines;
mod plane;
mod settings;
mod state;
mod texture;
mod world;
pub use state::*;

fn exit(control_flow: &mut ControlFlow) {
    logger::print();
    *control_flow = ControlFlow::Exit
}

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

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { ref event, .. } => {
            state.input.process_device_event(event);
        }
        Event::RedrawRequested(_) => {
            logger::measure_time("Update", || {
                state.update();
            });

            logger::measure_time("Render", || match state.render() {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => state.resize(winit::dpi::PhysicalSize::new(state.viewport.width, state.viewport.height)),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("Render error: {:?}", e),
            });

            fps += 1;
            if last_update.elapsed().as_millis() >= 1000 {
                window.set_title(format!("WGPU-RS: {} FPS", fps).as_str());
                last_update = Instant::now();
                fps = 0;
            }
        }
        Event::MainEventsCleared => {
            if state.viewport.valid {
                window.request_redraw();
            }
        }
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => exit(control_flow),
            WindowEvent::MouseInput {
                button,
                state: mouse_state,
                ..
            } => {
                &state.input.process_mouse_button(button, mouse_state);
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => exit(control_flow),
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::F),
                    ..
                } => {
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                    } else {
                        let fullscreen = Some(Fullscreen::Borderless(window.current_monitor()));
                        window.set_fullscreen(fullscreen.clone());
                    }
                }
                input => {
                    &state.input.process_key(input);
                }
            },
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
