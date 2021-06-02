use super::RunningState;
use super::{ActiveState, StateData, StateTypes};
use crate::{anti_aliasing, ui};

pub struct LoadingState {
    pub anti_aliasing: anti_aliasing::AntiAliasing,
}

impl StateTypes for LoadingState {
    fn initialize(&mut self, data: &mut StateData) {
        data.ui.load_op = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        ui::screens::loading::create(&mut data.ui.ui, "Creating World...");
    }

    fn update(&mut self, data: &mut StateData) -> Option<ActiveState> {
        Some(RunningState::from_loading(self, data))
    }

    fn render(&mut self, _data: &StateData, _frame: &wgpu::SwapChainTexture) {}

    fn resize(&mut self, data: &StateData) {
        self.anti_aliasing.resize(&data.device, data.window_width, data.winder_height);
    }
}

impl LoadingState {
    pub fn new(data: &StateData) -> ActiveState {
        let anti_aliasing = anti_aliasing::AntiAliasing::new(&data.device, &data.queue, data.window_width, data.winder_height);
        ActiveState::Loading(Self { anti_aliasing })
    }
}
