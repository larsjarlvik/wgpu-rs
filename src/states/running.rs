use super::{ActiveState, LoadingState, StateData, StateTypes};
use crate::{anti_aliasing, camera, ui, world};
use futures::executor::block_on;
use std::time::Instant;
use winit::event::VirtualKeyCode;

pub struct RunningState {
    world: world::World,
    anti_aliasing: anti_aliasing::AntiAliasing,
    viewport: camera::Viewport,
    start_time: Instant,
    fps: u32,
    current_fps: u32,
    last_update: Instant,
}

impl RunningState {
    pub fn from_loading(current: &mut LoadingState, data: &StateData) -> ActiveState {
        let viewport = camera::Viewport::new(&data.device, data.window_width, data.winder_height);
        let world = block_on(world::World::new(&data.device, &data.queue, &viewport));
        let anti_aliasing = std::mem::take(&mut current.anti_aliasing);
        let start_time = Instant::now();

        ActiveState::Running(Self {
            world,
            anti_aliasing,
            viewport,
            start_time,
            fps: 0,
            current_fps: 0,
            last_update: Instant::now(),
        })
    }
}

impl StateTypes for RunningState {
    fn initialize(&mut self, data: &mut StateData) {
        block_on(self.world.generate(&data.device, &data.queue, &self.viewport));
        data.ui.ui.set_widgets();
        data.ui.load_op = wgpu::LoadOp::Load;
    }

    fn update(&mut self, data: &mut StateData) -> Option<ActiveState> {
        if let Some(repeat) = data.input.keys.get(&VirtualKeyCode::G) {
            if !repeat {
                self.anti_aliasing.toggle(&data.device, &data.queue);
            }
        }

        ui::screens::status::create(
            &mut data.ui.ui,
            &ui::screens::status::Status {
                fps: self.fps,
                anti_aliasing: self.anti_aliasing.display().to_string(),
            },
        );

        self.viewport.update(&data.input, data.frame_time);
        self.world.update(&data.device, &data.queue, &self.viewport, self.start_time);
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

        self.current_fps += 1;
        if self.last_update.elapsed().as_millis() >= 1000 {
            self.last_update = Instant::now();
            self.fps = self.current_fps;
            self.current_fps = 0;
        }
    }

    fn resize(&mut self, data: &StateData) {
        self.viewport.resize(data.window_width, data.winder_height);
        self.anti_aliasing.resize(&data.device, data.window_width, data.winder_height);
        self.world.resize(&data.device, &self.viewport);
    }
}
