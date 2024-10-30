use crate::{machine::QueryableComponents, rom::RomManager};
use downcast_rs::DowncastSync;
use std::fmt::Debug;
use std::{any::Any, sync::Arc};

pub mod audio;
pub mod definitions;
pub mod display;
pub mod input;
pub mod memory;
pub mod processor;
pub mod schedulable;
pub mod snapshot;

// Basic supertrait for all components
pub trait Component: DowncastSync + Any + Send + Sync + 'static {
    fn reset(&mut self) {}
    fn query_components(&mut self, query: &QueryableComponents) {}
}

// An initializable component
pub trait FromConfig: Component + Sized {
    type Config: Debug;

    /// Make a new component from the config
    fn from_config(rom_manager: Arc<RomManager>, config: Self::Config) -> Self;
}
