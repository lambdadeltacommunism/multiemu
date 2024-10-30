use std::sync::Arc;

use crate::{
    component::{
        memory::MemoryTranslationTable, schedulable::SchedulableComponent, Component, FromConfig,
    },
    rom::RomManager,
};
use num::rational::Ratio;

#[derive(Debug)]
pub struct Chip8Timer {
    // The CPU will set this according to what the program wants
    pub delay_timer: u8,
}

impl Component for Chip8Timer {}

impl FromConfig for Chip8Timer {
    type Config = ();

    fn from_config(_rom_manager: Arc<RomManager>, _config: Self::Config) -> Self {
        Self { delay_timer: 0 }
    }
}

impl SchedulableComponent for Chip8Timer {
    fn tick_rate(&self) -> Ratio<u32> {
        Ratio::new(60, 1)
    }

    fn tick(&mut self, _: &MemoryTranslationTable) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
    }
}
