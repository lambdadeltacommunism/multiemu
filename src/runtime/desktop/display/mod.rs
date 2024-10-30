use crate::{config::GlobalConfig, runtime::RenderingBackendState};
use std::sync::{Arc, RwLock};
use winit::window::Window;

pub mod software;
pub mod vulkan;

pub trait WinitRenderBackendState: RenderingBackendState {
    fn new(window: Arc<Window>, global_config: Arc<RwLock<GlobalConfig>>) -> Self;
}
