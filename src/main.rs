use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Fullscreen,
    window::WindowBuilder,
};
mod anti_aliasing;
mod camera;
mod input;
mod logger;
mod model;
mod noise;
mod pipelines;
mod plane;
mod settings;
mod state;
mod states;
mod texture;
mod ui;
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
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut state = block_on(state::State::new(&window));
    let mut profiling = false;
    let mut valid_size = true;

    window.set_visible(true);
    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { ref event, .. } => {
            state.state_data.input.process_device_event(event);
        }
        Event::RedrawRequested(_) => {
            optick::next_frame();

            logger::measure_time("Update", || {
                state.update();
            });

            logger::measure_time("Render", || match state.render() {
                Ok(_) => {}
                Err(wgpu::SwapChainError::Lost) => state.resize(window.inner_size()),
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("Render error: {:?}", e),
            });
        }
        Event::MainEventsCleared => {
            if valid_size {
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
                &state.state_data.input.process_mouse_button(button, mouse_state);
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => exit(control_flow),
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::P),
                    ..
                } => {
                    if profiling {
                        optick::stop_capture("wgpu-profile");
                    } else {
                        optick::start_capture();
                    }
                    profiling = !profiling;
                }
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
                    &state.state_data.input.process_key(input);
                }
            },
            WindowEvent::Resized(physical_size) => {
                valid_size = physical_size.width > 0 && physical_size.height > 0;
                if valid_size {
                    state.resize(*physical_size);
                }
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        },
        _ => {}
    });
}
