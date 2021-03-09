use crate::{camera, fxaa, input::Input, world};
use std::time::Instant;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    pub viewport: camera::Viewport,
    world: world::World,
    fxaa: fxaa::Fxaa,
    start_time: Instant,
    last_frame: Instant,
    frame_time: Vec<f32>,
    input: Input,
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
                        | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device");

        // Camera
        let viewport = camera::Viewport::new(&device, size.width, size.height);
        let swap_chain = viewport.create_swap_chain(&device, &surface);

        // Drawing
        let fxaa = fxaa::Fxaa::new(&device, viewport.width, viewport.height);

        // World
        let world = world::World::new(&device, &queue, &viewport).await;

        Self {
            surface,
            device,
            queue,
            swap_chain,
            fxaa,
            viewport,
            world,
            input: Input::new(),
            start_time: Instant::now(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.viewport.resize(new_size.width, new_size.height);
        self.swap_chain = self.viewport.create_swap_chain(&self.device, &self.surface);
        self.fxaa = fxaa::Fxaa::new(&self.device, self.viewport.width, self.viewport.height);
        self.world.resize(&self.device, &self.viewport);
    }

    pub fn input(&mut self, event: &DeviceEvent) {
        &self.input.process_events(event);
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
        self.world.update(&self.device, &self.queue, &self.viewport, self.start_time);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
        {
            self.world.render(&mut encoder, &self.fxaa.texture_view);
            self.fxaa.render(&mut encoder, &frame.view);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
