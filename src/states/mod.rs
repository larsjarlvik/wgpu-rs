use crate::{camera, ui};
use enum_dispatch::enum_dispatch;

mod empty;
mod loading;
mod running;

pub use {empty::EmptyState, loading::LoadingState, running::RunningState};
pub struct StateData {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub viewport: camera::Viewport,
    pub ui: ui::Ui,
}

#[enum_dispatch]
pub enum ActiveState {
    Empty(EmptyState),
    Loading(LoadingState),
    Running(RunningState),
}

#[enum_dispatch(ActiveState)]
pub trait StateTypes {
    fn initialize(&mut self, data: &mut StateData);
    fn update(&mut self, data: &StateData) -> Option<ActiveState>;
    fn render(&mut self, data: &StateData, target: &wgpu::SwapChainTexture);
    fn resize(&mut self, data: &StateData);
}
