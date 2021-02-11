use crate::{camera, deferred, fxaa, input::Input, settings, world};
use std::time::Instant;
use winit::{event::*, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    cameras: camera::Cameras,
    world: world::World,
    deferred_render: deferred::DeferredRender,
    fxaa: fxaa::Fxaa,
    start_time: Instant,
    last_frame: Instant,
    frame_time: Vec<f32>,
    input: Input,
    pub size: winit::dpi::PhysicalSize<u32>,
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
                    features: wgpu::Features::SAMPLED_TEXTURE_BINDING_ARRAY | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device");

        // Swap chain
        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: settings::COLOR_TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        // Camera
        let cameras = camera::Cameras::new(&device, &swap_chain_desc);

        // Drawing
        let deferred_render = deferred::DeferredRender::new(&device, &swap_chain_desc, &cameras);
        let fxaa = fxaa::Fxaa::new(&device, &swap_chain_desc);

        // World
        let world = world::World::new(&device, &swap_chain_desc, &queue, &cameras).await;

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_desc,
            size,
            deferred_render,
            fxaa,
            cameras,
            world,
            input: Input::new(),
            start_time: Instant::now(),
            last_frame: Instant::now(),
            frame_time: Vec::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swap_chain_desc.width = new_size.width;
        self.swap_chain_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.swap_chain_desc);
        self.cameras.resize(&self.swap_chain_desc);
        self.deferred_render = deferred::DeferredRender::new(&self.device, &self.swap_chain_desc, &self.cameras);
        self.fxaa = fxaa::Fxaa::new(&self.device, &self.swap_chain_desc);
        self.world.resize(&self.device, &self.swap_chain_desc, &self.cameras);
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
        let avg = self.frame_time();
        self.cameras.update_camera(&self.queue, &self.input, avg);
        self.input.after_update();
        self.world.update(&self.device, &self.queue, &self.cameras, self.start_time);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        // Water reflection pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
        {
            self.world.update_bundle(&self.device, &self.cameras.refraction_cam);
            self.world.render(&mut encoder, &self.deferred_render.target, true);

            let deferred_bundle = self.deferred_render.get_render_bundle(&self.device, &self.cameras.refraction_cam);
            self.deferred_render.render(&mut encoder, &self.world.data.water.reflection_texture_view, &self.world.data.water.reflection_depth_texture_view, &deferred_bundle);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        // Water refraction pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
        {
            self.world.update_bundle(&self.device, &self.cameras.reflection_cam);
            self.world.render(&mut encoder, &self.deferred_render.target, false);
            let deferred_bundle = self.deferred_render.get_render_bundle(&self.device, &self.cameras.refraction_cam);
            self.deferred_render.render(&mut encoder, &self.world.data.water.refraction_texture_view, &self.world.data.water.refraction_depth_texture_view, &deferred_bundle);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("main") });
        {
            // Main render pass
            self.world.update_bundle(&self.device, &self.cameras.eye_cam);
            self.world.render(&mut encoder, &self.deferred_render.target, true);
            let deferred_bundle = self.deferred_render.get_render_bundle(&self.device, &self.cameras.refraction_cam);
            self.deferred_render.render(&mut encoder, &self.fxaa.texture_view, &self.fxaa.depth_texture_view, &deferred_bundle);
            self.world.render_water(&mut encoder, &self.fxaa.texture_view, &self.fxaa.depth_texture_view);

            // Post processing
            self.fxaa.render(&mut encoder, &frame.view);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
