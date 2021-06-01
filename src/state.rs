use crate::{camera, input::Input, states, ui};
use std::time::Instant;
use winit::window::Window;

pub struct State {
    pub viewport: camera::Viewport,
    pub input: Input,
    pub state: states::StateMachine,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    ui: ui::UI,
    last_frame: Instant,
    frame_time: Vec<f32>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        // Device
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | wgpu::Features::DEPTH_CLAMPING,
                    limits: wgpu::Limits {
                        max_bind_groups: 7,
                        max_sampled_textures_per_shader_stage: 32,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .expect("Failed to request device");

        // Camera
        let viewport = camera::Viewport::new(&device, size.width, size.height);
        let swap_chain = viewport.create_swap_chain(&device, &surface);

        // UI
        let ui = ui::UI::new(&device, &viewport);
        window.request_redraw();

        // State
        let state = states::StateMachine::new();

        Self {
            surface,
            device,
            queue,
            swap_chain,
            viewport,
            ui,
            input: Input::new(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
            state,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            self.viewport.valid = false;
            return;
        }

        self.viewport.valid = true;
        self.viewport.resize(new_size.width, new_size.height);
        self.swap_chain = self.viewport.create_swap_chain(&self.device, &self.surface);
        self.ui.resize(&self.viewport);

        match &mut self.state.current {
            states::GameState::Loading(s) => {
                s.anti_aliasing.resize(&self.device, &self.viewport);
            }
            states::GameState::Running(s) => {
                s.anti_aliasing.resize(&self.device, &self.viewport);
                s.world.resize(&self.device, &self.viewport);
            }
            _ => {}
        }
    }

    fn frame_time(&mut self) -> f32 {
        let avg_count = 30;
        self.frame_time.push(self.last_frame.elapsed().as_micros() as f32 / 1000.0);
        self.last_frame = Instant::now();

        if self.frame_time.len() > avg_count {
            self.frame_time.drain((self.frame_time.len() - avg_count)..);
        }
        self.frame_time.iter().sum::<f32>() / self.frame_time.len() as f32
    }

    pub fn update(&mut self) {
        let frame_time = self.frame_time();
        self.viewport.update(&self.input, frame_time);
        self.input.after_update();
        self.ui.update(&self.state);
        self.state.update(&self.device, &self.queue, &self.viewport);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;
        let device = &mut self.device;
        let queue = &mut self.queue;

        self.state.render(device, queue, &frame);

        self.ui.render(&device, &queue, &frame.view);
        Ok(())
    }
}
