use crate::rom::RomId;
use crate::rom::RomManager;
use crate::{component::definitions::chip8::display::Chip8DisplayConfig, machine::Machine};
use crate::{
    component::definitions::chip8::processor::Chip8Processor,
    component::definitions::chip8::processor::Chip8ProcessorConfig, task::generic::GenericTask,
    task::processor::ProcessorTask,
};
use crate::{
    component::definitions::{chip8::CHIP8_FONT, misc::plain_memory::PlainMemoryInitialContents},
    runtime::RenderingBackend,
};
use crate::{
    component::{
        definitions::{
            chip8::{audio::Chip8Audio, display::Chip8Display, timer::Chip8Timer, Chip8Kind},
            misc::plain_memory::{PlainMemory, PlainMemoryConfig},
        },
        display::DisplayComponent,
    },
    task::processor::ProcessorTaskConfig,
};
use num::rational::Ratio;
use std::sync::Arc;

pub fn other_chip8<R: RenderingBackend>(
    rom_manager: Arc<RomManager>,
    user_specified_roms: Vec<RomId>,
    rendering_state: &mut <R as RenderingBackend>::RuntimeState,
) -> Machine<R>
where
    Chip8Display: DisplayComponent<R>,
{
    Machine::build(rom_manager, rendering_state)
        .component::<Chip8Processor>(
            "processor",
            Chip8ProcessorConfig {
                frequency: Ratio::new(700, 1),
                kind: Chip8Kind::Chip8,
            },
        )
        .insert_schedule::<ProcessorTask<_>>(ProcessorTaskConfig {
            initial_program_pointer: 0x200,
        })
        .with_gamepad()
        .finalize_component()
        .component::<PlainMemory>(
            "system_memory",
            PlainMemoryConfig {
                readable: true,
                writable: true,
                max_word_size: 2,
                read_cycle_penalty_calculator: |_, _| 0,
                write_cycle_penalty_calculator: |_, _| 0,
                assigned_range: 0x000..0x200,
                initial_contents: PlainMemoryInitialContents::Array {
                    value: bytemuck::cast_slice(&CHIP8_FONT),
                    offset: 0x000,
                },
            },
        )
        .with_memory_map()
        .finalize_component()
        .component::<PlainMemory>(
            "work_memory",
            PlainMemoryConfig {
                readable: true,
                writable: true,
                max_word_size: 2,
                read_cycle_penalty_calculator: |_, _| 0,
                write_cycle_penalty_calculator: |_, _| 0,
                assigned_range: 0x200..0x1000,
                initial_contents: PlainMemoryInitialContents::Rom {
                    rom_id: user_specified_roms[0],
                    offset: 0x200,
                },
            },
        )
        .with_memory_map()
        .finalize_component()
        .component::<Chip8Display>(
            "display",
            Chip8DisplayConfig {
                kind: Chip8Kind::Chip8,
            },
        )
        .with_displayable()
        .insert_schedule_default::<GenericTask<_>>()
        .finalize_component()
        .component_default::<Chip8Timer>("timer")
        .insert_schedule_default::<GenericTask<_>>()
        .finalize_component()
        .component_default::<Chip8Audio>("audio")
        .insert_schedule_default::<GenericTask<_>>()
        .finalize_component()
        .finalize_machine()
}
