use crate::ui;

use super::{ActiveState, LoadingState, StateData, StateTypes};

pub struct EmptyState {}

impl StateTypes for EmptyState {
    fn initialize(&mut self, data: &mut StateData) {
        data.ui.load_op = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        ui::screens::loading::create(&mut data.ui.ui, "Loading...");
    }

    fn update(&mut self, data: &mut StateData) -> Option<ActiveState> {
        Some(LoadingState::new(data))
    }

    fn render(&mut self, _data: &StateData, _target: &wgpu::SwapChainTexture) {}
    fn resize(&mut self, _data: &StateData) {}
}
