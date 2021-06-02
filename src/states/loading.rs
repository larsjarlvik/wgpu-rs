use super::RunningState;
use super::{ActiveState, StateData, StateTypes};
use crate::anti_aliasing;
use conrod_core::*;

pub struct LoadingState {
    pub anti_aliasing: anti_aliasing::AntiAliasing,
}

impl StateTypes for LoadingState {
    fn initialize(&mut self, data: &mut StateData) {
        let mut widgets = data.ui.ui.set_widgets();

        data.ui.load_op = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        widget::Text::new("Creating World...")
            .middle_of(widgets.window)
            .color(color::WHITE)
            .font_size(32)
            .set(data.ui.ids.title, &mut widgets);
    }

    fn update(&mut self, data: &StateData) -> Option<ActiveState> {
        Some(RunningState::from_loading(self, data))
    }

    fn render(&mut self, _data: &StateData, _frame: &wgpu::SwapChainTexture) {}

    fn resize(&mut self, data: &StateData) {
        self.anti_aliasing.resize(&data.device, &data.viewport);
    }
}

impl LoadingState {
    pub fn new(data: &StateData) -> ActiveState {
        let anti_aliasing = anti_aliasing::AntiAliasing::new(&data.device, &data.queue, &data.viewport);
        ActiveState::Loading(Self { anti_aliasing })
    }
}
