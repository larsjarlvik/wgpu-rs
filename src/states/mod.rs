use crate::{input::Input, ui};
use enum_dispatch::enum_dispatch;

mod empty;
mod loading;
mod running;

pub use {empty::EmptyState, loading::LoadingState, running::RunningState};
pub struct StateData {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub input: Input,
    pub window_width: u32,
    pub winder_height: u32,
    pub ui: ui::Ui,
    pub frame_time: f32,
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
    fn update(&mut self, data: &mut StateData) -> Option<ActiveState>;
    fn render(&mut self, data: &StateData, target: &wgpu::SwapChainTexture);
    fn resize(&mut self, data: &StateData);
}
