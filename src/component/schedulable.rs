use super::{memory::MemoryTranslationTable, Component};
use num::rational::Ratio;

pub trait SchedulableComponent: Component {
    fn tick_rate(&self) -> Ratio<u32>;

    // Takes in the ticker resolution and returns how many times it needs to run in how many of this resolution
    fn tick(&mut self, memory_translation_table: &MemoryTranslationTable);
}
