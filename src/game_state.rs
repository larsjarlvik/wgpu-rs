use crate::{anti_aliasing, camera, world};
use futures::executor::block_on;
use std::time::Instant;

pub enum GameState {
    Empty,
    Loading(LoadingState),
    Running(RunningState),
}

pub struct LoadingState {
    pub anti_aliasing: anti_aliasing::AntiAliasing,
}

impl LoadingState {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> GameState {
        let anti_aliasing = anti_aliasing::AntiAliasing::new(device, queue, viewport);
        GameState::Loading(Self { anti_aliasing })
    }
}

pub struct RunningState {
    pub world: world::World,
    pub anti_aliasing: anti_aliasing::AntiAliasing,
    pub start_time: Instant,
}

impl RunningState {
    pub fn new(initialized: &mut LoadingState, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> GameState {
        let mut world = block_on(world::World::new(device, queue, viewport));
        let anti_aliasing = std::mem::take(&mut initialized.anti_aliasing);

        block_on(world.generate(device, queue, viewport));

        let start_time = Instant::now();
        GameState::Running(Self {
            world,
            anti_aliasing,
            start_time,
        })
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) {
        self.world.update(device, queue, viewport, self.start_time);
    }
}

pub struct StateMachine {
    pub current: GameState,
    pub fresh: bool,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: GameState::Empty,
            fresh: true,
        }
    }

    pub fn next(state: GameState) -> Self {
        println!("next");
        Self {
            current: state,
            fresh: true,
        }
    }
}
