use crate::{component::memory::MemoryTranslationTable, task::Task};
use num::rational::Ratio;
use std::{sync::Arc, time::Duration};

pub mod single;

pub trait Executor {
    fn new(
        tasks: Vec<(Ratio<u32>, Box<dyn Task>)>,
        memory_translation_table: Arc<MemoryTranslationTable>,
    ) -> Self;
    fn run(&mut self, period: Duration);
}
