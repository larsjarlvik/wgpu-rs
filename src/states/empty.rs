use super::{ActiveState, LoadingState, StateData, StateTypes};
use conrod_core::*;

pub struct EmptyState {}

impl StateTypes for EmptyState {
    fn initialize(&mut self, data: &mut StateData) {
        let mut widgets = data.ui.ui.set_widgets();

        data.ui.load_op = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        widget::Text::new("Loading...")
            .middle_of(widgets.window)
            .color(color::WHITE)
            .font_size(32)
            .set(data.ui.ids.title, &mut widgets);
    }

    fn update(&mut self, data: &StateData) -> Option<ActiveState> {
        Some(LoadingState::new(data))
    }

    fn render(&mut self, _data: &StateData, _target: &wgpu::SwapChainTexture) {}
    fn resize(&mut self, _data: &StateData) {}
}
