use super::{InitializeableTask, Task};
use crate::component::{
    memory::MemoryTranslationTable, processor::ProcessorComponent,
    schedulable::SchedulableComponent,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
struct TaskState {
    program_pointer: usize,
}

#[derive(Debug)]
pub struct ProcessorTaskConfig {
    pub initial_program_pointer: usize,
}

pub struct ProcessorTask<C: ProcessorComponent> {
    program_pointer: usize,
    component: Arc<Mutex<C>>,
}

impl<C: ProcessorComponent> Task for ProcessorTask<C> {
    fn tick(&mut self, batch_size: u32, memory_translation_table: &MemoryTranslationTable) {
        let mut component = self.component.lock().unwrap();

        for _ in 0..batch_size {
            // Tick
            component.tick(memory_translation_table);

            if !component.should_execution_occur() {
                continue;
            }

            // Fetch / decode
            let (instruction, size) = component
                .decompile(self.program_pointer, memory_translation_table)
                .unwrap();

            tracing::debug!(
                "Instruction: {:x?} decoded from address: 0x{:x}",
                instruction,
                self.program_pointer
            );

            self.program_pointer = self.program_pointer.wrapping_add(size as usize);

            // Execute
            component
                .interpret(
                    &mut self.program_pointer,
                    instruction,
                    memory_translation_table,
                )
                .unwrap();
        }
    }

    fn save(&mut self) -> rmpv::Value {
        let state = TaskState {
            program_pointer: self.program_pointer,
        };

        rmpv::ext::to_value(&state).unwrap()
    }

    fn load(&mut self, state: rmpv::Value) {
        let state = rmpv::ext::from_value::<TaskState>(state).unwrap();

        self.program_pointer = state.program_pointer;
    }
}

impl<C: ProcessorComponent> InitializeableTask<C> for ProcessorTask<C> {
    type Config = ProcessorTaskConfig;

    fn new(component: Arc<Mutex<C>>, config: Self::Config) -> Self {
        Self {
            program_pointer: config.initial_program_pointer,
            component,
        }
    }
}
