use super::{GameState, LoadingState, State};
use crate::{anti_aliasing, camera, world};
use futures::executor::block_on;
use std::time::Instant;

pub struct RunningState {
    pub world: world::World,
    pub anti_aliasing: anti_aliasing::AntiAliasing,
    pub start_time: Instant,
}

impl State for RunningState {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Option<GameState> {
        self.world.update(device, queue, viewport, self.start_time);
        None
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &wgpu::SwapChainTexture) {
        let aa = &mut self.anti_aliasing;
        let world = &self.world;

        aa.execute(&device, &queue, &frame.view, |color_target, depth_target| {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
            world.render(&mut encoder, &color_target, &depth_target);
            {
                optick::event!("submit");
                queue.submit(std::iter::once(encoder.finish()));
            }
        });
    }
}

impl RunningState {
    pub fn new(current: &mut LoadingState, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> GameState {
        let mut world = block_on(world::World::new(device, queue, viewport));

        let anti_aliasing = std::mem::take(&mut current.anti_aliasing);
        block_on(world.generate(device, queue, viewport));
        let start_time = Instant::now();

        GameState::Running(Self {
            world,
            anti_aliasing,
            start_time,
        })
    }
}
