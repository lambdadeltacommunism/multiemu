use super::Machine;
use crate::{
    component::{definitions::chip8::display::Chip8Display, display::DisplayComponent},
    rom::{
        AtariSystem, GameSystem, NintendoSystem, OtherSystem, RomId, RomManager, SegaSystem,
        SonySystem,
    },
    runtime::RenderingBackend,
};
use atari_atari2600::atari_atari2600;
use other_chip8::other_chip8;
use std::sync::Arc;

mod atari_atari2600;
mod other_chip8;
mod other_superchip8;
mod sega_gamegear;
mod sony_playstation;

pub fn construct_machine<R: RenderingBackend>(
    game_system: GameSystem,
    rom_manager: Arc<RomManager>,
    user_specified_roms: Vec<RomId>,
    rendering_state: &mut <R as RenderingBackend>::RuntimeState,
) -> Machine<R>
where
    Chip8Display: DisplayComponent<R>,
{
    match game_system {
        GameSystem::Nintendo(NintendoSystem::GameBoy) => todo!(),
        GameSystem::Nintendo(NintendoSystem::GameBoyColor) => todo!(),
        GameSystem::Nintendo(NintendoSystem::GameBoyAdvance) => todo!(),
        GameSystem::Nintendo(NintendoSystem::SuperNintendoEntertainmentSystem) => todo!(),
        GameSystem::Nintendo(NintendoSystem::NintendoEntertainmentSystem) => todo!(),
        GameSystem::Nintendo(NintendoSystem::Nintendo64) => todo!(),
        GameSystem::Sega(SegaSystem::GameGear) => todo!(),
        GameSystem::Sega(SegaSystem::Genesis) => todo!(),
        GameSystem::Sega(SegaSystem::MasterSystem) => todo!(),
        GameSystem::Sony(SonySystem::Playstation) => todo!(),
        GameSystem::Atari(AtariSystem::Atari2600) => {
            atari_atari2600::<R>(rom_manager, user_specified_roms, rendering_state)
        }
        GameSystem::Other(OtherSystem::Chip8) => {
            other_chip8::<R>(rom_manager, user_specified_roms, rendering_state)
        }
        GameSystem::Other(OtherSystem::SuperChip8) => todo!(),
        _ => {
            unimplemented!("This system is unlikely to ever be supported by this emulator")
        }
    }
}
