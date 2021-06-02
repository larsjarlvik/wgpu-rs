use crate::states::StateTypes;
use crate::{camera, input::Input, states, ui};
use std::time::Instant;
use winit::window::Window;

pub struct State {
    pub input: Input,
    pub current: states::ActiveState,
    pub state_data: states::StateData,
    pub fresh: bool,
    surface: wgpu::Surface,
    swap_chain: wgpu::SwapChain,
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
        let ui = ui::Ui::new(&device, &viewport);
        window.request_redraw();

        // State
        let mut state_data = states::StateData {
            device,
            queue,
            viewport,
            ui,
        };
        let mut current = states::ActiveState::Empty(states::EmptyState {});
        current.initialize(&mut state_data);
        let fresh = false;

        Self {
            surface,
            swap_chain,
            input: Input::new(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
            state_data,
            current,
            fresh,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            self.state_data.viewport.valid = false;
            return;
        }

        self.state_data.viewport.valid = true;
        self.state_data.viewport.resize(new_size.width, new_size.height);
        self.swap_chain = self.state_data.viewport.create_swap_chain(&self.state_data.device, &self.surface);
        self.state_data.ui.resize(&self.state_data.viewport);
        self.current.resize(&self.state_data);
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
        self.state_data.viewport.update(&self.input, frame_time);

        if self.fresh {
            if let Some(next_state) = self.current.update(&self.state_data) {
                self.current = next_state;
                self.current.initialize(&mut self.state_data);
                self.fresh = false;
                return;
            }
        }

        self.fresh = true;
        self.input.after_update();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        self.current.render(&self.state_data, &frame);
        self.state_data
            .ui
            .render(&self.state_data.device, &self.state_data.queue, &frame.view);
        Ok(())
    }
}
