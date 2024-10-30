use crate::component::{memory::MemoryTranslationTable, schedulable::SchedulableComponent};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub mod generic;
pub mod processor;

/// Trait that wraps a [ScheduableComponent] to provide more functionality and handle batching
pub trait Task: Send + Sync + 'static {
    fn tick(&mut self, batch_size: u32, memory_translation_table: &MemoryTranslationTable);

    fn save(&mut self) -> rmpv::Value;
    fn load(&mut self, state: rmpv::Value);
}

pub trait InitializeableTask<C: SchedulableComponent>: Task + Sized {
    type Config: Debug;

    fn new(component: Arc<Mutex<C>>, config: Self::Config) -> Self;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TaskOrdering {
    Before,
    After,
}
