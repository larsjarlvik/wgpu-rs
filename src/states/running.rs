use super::{ActiveState, LoadingState, StateData, StateTypes};
use crate::{anti_aliasing, world};
use futures::executor::block_on;
use std::time::Instant;

pub struct RunningState {
    pub world: world::World,
    pub anti_aliasing: anti_aliasing::AntiAliasing,
    pub start_time: Instant,
}

impl RunningState {
    pub fn from_loading(current: &mut LoadingState, data: &StateData) -> ActiveState {
        let world = block_on(world::World::new(&data.device, &data.queue, &data.viewport));
        let anti_aliasing = std::mem::take(&mut current.anti_aliasing);
        let start_time = Instant::now();

        ActiveState::Running(Self {
            world,
            anti_aliasing,
            start_time,
        })
    }
}

impl StateTypes for RunningState {
    fn initialize(&mut self, data: &mut StateData) {
        data.ui.ui.set_widgets();
        data.ui.load_op = wgpu::LoadOp::Load;
        block_on(self.world.generate(&data.device, &data.queue, &data.viewport));
    }

    fn update(&mut self, data: &StateData) -> Option<ActiveState> {
        self.world.update(&data.device, &data.queue, &data.viewport, self.start_time);
        None
    }

    fn render(&mut self, data: &StateData, target: &wgpu::SwapChainTexture) {
        let aa = &mut self.anti_aliasing;
        let world = &self.world;

        aa.execute(&data.device, &data.queue, &target.view, |color_target, depth_target| {
            let mut encoder = data
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("refraction") });
            world.render(&mut encoder, &color_target, &depth_target);
            {
                optick::event!("submit");
                data.queue.submit(std::iter::once(encoder.finish()));
            }
        });
    }

    fn resize(&mut self, data: &StateData) {
        self.anti_aliasing.resize(&data.device, &data.viewport);
        self.world.resize(&data.device, &data.viewport);
    }
}
