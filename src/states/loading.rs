use super::{GameState, RunningState, State};
use crate::{anti_aliasing, camera};

pub struct LoadingState {
    pub anti_aliasing: anti_aliasing::AntiAliasing,
}

impl State for LoadingState {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Option<GameState> {
        Some(RunningState::new(self, &device, &queue, &viewport))
    }

    fn render(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _frame: &wgpu::SwapChainTexture) {}
}

impl LoadingState {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> GameState {
        let anti_aliasing = anti_aliasing::AntiAliasing::new(device, queue, viewport);
        GameState::Loading(Self { anti_aliasing })
    }
}
