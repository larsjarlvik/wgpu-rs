use crate::camera;
mod empty;
mod loading;
mod running;

pub use {empty::EmptyState, loading::LoadingState, running::RunningState};

pub trait State {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Option<GameState>;
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &wgpu::SwapChainTexture);
}

pub enum GameState {
    Empty(EmptyState),
    Loading(LoadingState),
    Running(RunningState),
}

pub struct StateMachine {
    pub current: GameState,
    pub fresh: bool,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: GameState::Empty(EmptyState {}),
            fresh: false,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) {
        if self.fresh {
            if let Some(next_state) = match &mut self.current {
                GameState::Empty(s) => s.update(device, queue, viewport),
                GameState::Loading(s) => s.update(device, queue, viewport),
                GameState::Running(s) => s.update(device, queue, viewport),
            } {
                self.current = next_state;
                self.fresh = false;
                return;
            }
        }
        self.fresh = true;
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &wgpu::SwapChainTexture) {
        match &mut self.current {
            GameState::Empty(s) => s.render(device, queue, frame),
            GameState::Loading(s) => s.render(device, queue, frame),
            GameState::Running(s) => s.render(device, queue, frame),
        }
    }
}
