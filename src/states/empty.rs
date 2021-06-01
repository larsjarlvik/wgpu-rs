use super::{GameState, LoadingState, State};
use crate::camera;

pub struct EmptyState {}

impl State for EmptyState {
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport) -> Option<GameState> {
        Some(LoadingState::new(device, queue, viewport))
    }

    fn render(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, _frame: &wgpu::SwapChainTexture) {}
}
