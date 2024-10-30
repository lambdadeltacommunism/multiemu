use crate::machine::Machine;
use crate::rom::RomId;
use crate::rom::RomManager;
use crate::runtime::RenderingBackend;
use crate::task::processor::ProcessorTask;
use crate::{
    component::definitions::misc::processor::m6502::{M6502Config, M6502},
    task::processor::ProcessorTaskConfig,
};
use num::rational::Ratio;
use std::sync::Arc;

pub fn atari_atari2600<R: RenderingBackend>(
    rom_manager: Arc<RomManager>,
    user_specified_roms: Vec<RomId>,
    rendering_state: &mut <R as RenderingBackend>::RuntimeState,
) -> Machine<R> {
    Machine::build(rom_manager, rendering_state)
        .component::<M6502>(
            "processor",
            M6502Config {
                frequency: Ratio::new(1193182, 1),
            },
        )
        .insert_schedule::<ProcessorTask<_>>(ProcessorTaskConfig {
            initial_program_pointer: 0x0000,
        })
        .finalize_component()
        .finalize_machine()
}
