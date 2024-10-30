use super::Chip8Kind;
use crate::{
    component::{
        memory::MemoryTranslationTable, schedulable::SchedulableComponent,
        snapshot::SnapshotableComponent, Component, FromConfig,
    },
    rom::RomManager,
};
use nalgebra::{DMatrix, Point2};
use num::rational::Ratio;
use palette::Srgba;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(desktop)]
mod desktop;
#[cfg(desktop)]
use desktop::vulkan::VulkanState;

mod software;
use software::SoftwareState;

#[non_exhaustive]
enum InternalState {
    #[cfg(desktop)]
    Vulkan(VulkanState),
    Software(SoftwareState),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chip8DisplaySnapshot {
    screen_buffer: DMatrix<Srgba<u8>>,
}

pub struct Chip8Display {
    config: Chip8DisplayConfig,
    state: Option<InternalState>,
}

impl Chip8Display {
    pub fn draw_sprite(&mut self, position: Point2<u8>, sprite: &[u8]) -> bool {
        tracing::debug!(
            "Drawing sprite at position {} of dimensions 8x{}",
            position,
            sprite.len()
        );

        let position = match self.config.kind {
            Chip8Kind::Chip8 | Chip8Kind::Chip48 => Point2::new(position.x % 63, position.y % 31),
            Chip8Kind::SuperChip8 => todo!(),
            _ => todo!(),
        };

        match &mut self.state {
            #[cfg(desktop)]
            Some(InternalState::Vulkan(vulkan_state)) => vulkan_state.draw_sprite(position, sprite),
            Some(InternalState::Software(software_state)) => {
                software_state.draw_sprite(position, sprite)
            }
            _ => panic!("Internal state not initialized"),
        }
    }

    pub fn clear_display(&mut self) {
        tracing::debug!("Clearing display");

        match &mut self.state {
            #[cfg(desktop)]
            Some(InternalState::Vulkan(vulkan_state)) => vulkan_state.clear_display(),
            Some(InternalState::Software(software_state)) => software_state.clear_display(),
            _ => panic!("Internal state not initialized"),
        }
    }
}

impl Component for Chip8Display {}

impl SnapshotableComponent for Chip8Display {
    fn save_snapshot(&mut self) -> rmpv::Value {
        let display_buffer = match &mut self.state {
            #[cfg(desktop)]
            Some(InternalState::Vulkan(vulkan_state)) => vulkan_state.get_display_buffer(),
            Some(InternalState::Software(software_state)) => software_state.get_display_buffer(),
            _ => panic!("Internal state not initialized"),
        };

        rmpv::ext::to_value(Chip8DisplaySnapshot {
            screen_buffer: display_buffer,
        })
        .unwrap()
    }

    fn load_snapshot(&mut self, state: rmpv::Value) {
        let snapshot: Chip8DisplaySnapshot = rmpv::ext::from_value(state).unwrap();

        match &mut self.state {
            #[cfg(desktop)]
            Some(InternalState::Vulkan(vulkan_state)) => {
                vulkan_state.set_screen_buffer(snapshot.screen_buffer);
            }
            Some(InternalState::Software(software_state)) => {
                software_state.set_screen_buffer(snapshot.screen_buffer);
            }
            _ => panic!("Internal state not initialized"),
        }
    }
}

#[derive(Debug)]
pub struct Chip8DisplayConfig {
    pub kind: Chip8Kind,
}

impl FromConfig for Chip8Display {
    type Config = Chip8DisplayConfig;

    fn from_config(_rom_manager: Arc<RomManager>, config: Self::Config) -> Self {
        Chip8Display {
            config,
            state: None,
        }
    }
}

trait Chip8DisplayImplementation {
    fn draw_sprite(&mut self, position: Point2<u8>, sprite: &[u8]) -> bool;
    fn clear_display(&mut self);
    fn get_display_buffer(&mut self) -> DMatrix<Srgba<u8>>;
    fn set_screen_buffer(&mut self, buffer: DMatrix<Srgba<u8>>);
    fn commit_display(&mut self);
}

impl SchedulableComponent for Chip8Display {
    fn tick_rate(&self) -> Ratio<u32> {
        // Chip8 waits after draw until vblank
        Ratio::new(60, 1)
    }

    fn tick(&mut self, _memory_translation_table: &MemoryTranslationTable) {
        match &mut self.state {
            #[cfg(desktop)]
            Some(InternalState::Vulkan(vulkan_state)) => {
                vulkan_state.commit_display();
            }
            Some(InternalState::Software(software_state)) => {
                software_state.commit_display();
            }
            _ => panic!("Internal state not initialized"),
        }
    }
}
