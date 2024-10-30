use std::sync::Arc;

use crate::{
    component::{
        audio::AudioComponent, memory::MemoryTranslationTable, schedulable::SchedulableComponent,
        Component, FromConfig,
    },
    rom::RomManager,
};
use num::rational::Ratio;

pub struct Chip8Audio {
    // The CPU will set this according to what the program wants
    pub sound_timer: u8,
}

impl Component for Chip8Audio {}

impl FromConfig for Chip8Audio {
    type Config = ();

    fn from_config(_rom_manager: Arc<RomManager>, _config: Self::Config) -> Self {
        Self { sound_timer: 0 }
    }
}

impl SchedulableComponent for Chip8Audio {
    fn tick_rate(&self) -> Ratio<u32> {
        Ratio::new(60, 1)
    }

    fn tick(&mut self, _: &MemoryTranslationTable) {
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }
}

impl AudioComponent for Chip8Audio {}
