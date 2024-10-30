use super::{InitializeableTask, Task};
use crate::component::{memory::MemoryTranslationTable, schedulable::SchedulableComponent};
use std::sync::{Arc, Mutex};

pub struct GenericTask<C: SchedulableComponent> {
    component: Arc<Mutex<C>>,
}

impl<C: SchedulableComponent> Task for GenericTask<C> {
    fn tick(&mut self, batch_size: u32, memory_translation_table: &MemoryTranslationTable) {
        let mut component = self.component.lock().unwrap();

        for _ in 0..batch_size {
            component.tick(memory_translation_table);
        }
    }

    fn load(&mut self, _state: rmpv::Value) {}

    fn save(&mut self) -> rmpv::Value {
        rmpv::Value::Nil
    }
}

impl<C: SchedulableComponent> InitializeableTask<C> for GenericTask<C> {
    type Config = ();

    fn new(component: Arc<Mutex<C>>, _: Self::Config) -> Self {
        Self { component }
    }
}
