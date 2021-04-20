use crate::{anti_aliasing, camera, input::Input, world};
use std::time::Instant;
use winit::window::Window;

pub struct State {
    pub viewport: camera::Viewport,
    pub input: Input,
    pub anti_aliasing: anti_aliasing::AntiAliasing,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    world: world::World,
    start_time: Instant,
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
        let world = world::World::new(&device, &queue, &viewport).await;
        let anti_aliasing = anti_aliasing::AntiAliasing::new(&device, &queue, &viewport);

        Self {
            surface,
            device,
            anti_aliasing,
            queue,
            swap_chain,
            viewport,
            world,
            input: Input::new(),
            start_time: Instant::now(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
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
        self.anti_aliasing = anti_aliasing::AntiAliasing::new(&self.device, &self.queue, &self.viewport);
        self.world.resize(&self.device, &self.viewport);
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

        let device = &mut self.device;
        let world = &mut self.world;
        let queue = &mut self.queue;
        self.anti_aliasing.execute(&device, &queue, &frame.view, |target| {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
            world.render(&mut encoder, &target);
            queue.submit(std::iter::once(encoder.finish()));
        });

        Ok(())
    }
}
