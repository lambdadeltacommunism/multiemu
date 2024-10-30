use super::{audio::Chip8Audio, display::Chip8Display, timer::Chip8Timer, Chip8Kind};
use crate::{
    component::{
        input::InputComponent,
        memory::MemoryTranslationTable,
        processor::{InstructionDecompilingError, ProcessorComponent},
        schedulable::SchedulableComponent,
        snapshot::SnapshotableComponent,
        Component, FromConfig,
    },
    input::{keyboard::KeyboardInput, EmulatedGamepad, Input},
    machine::QueryableComponents,
    rom::RomManager,
};
use arrayvec::ArrayVec;
use decode::decode_instruction;
use input::Chip8Key;
use instruction::{Chip8InstructionSet, Register};
use num::rational::Ratio;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

mod decode;
mod input;
mod instruction;
mod interpret;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    Normal,
    AwaitingKeyPress { register: Register },
    // KeyQuery does not return on key press but on key release, contrary to some documentation
    AwaitingKeyRelease { register: Register, key: Chip8Key },
}

// This is extremely complex because the chip8 cpu has a lot of non cpu machinery

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Chip8ProcessorRegisters {
    work_registers: [u8; 16],
    index: u16,
}

#[derive(Debug)]
pub struct Chip8ProcessorConfig {
    pub frequency: Ratio<u32>,
    pub kind: Chip8Kind,
}

pub struct ImportedComponents {
    pub display: Arc<Mutex<Chip8Display>>,
    pub timer: Arc<Mutex<Chip8Timer>>,
    pub audio: Arc<Mutex<Chip8Audio>>,
}

/// The chip8 cpu is not only a cpu but a display controller btw
pub struct Chip8Processor {
    config: Chip8ProcessorConfig,
    stack: ArrayVec<u16, 16>,
    registers: Chip8ProcessorRegisters,
    imported: Option<ImportedComponents>,
    controller: Option<Arc<EmulatedGamepad>>,
    execution_state: ExecutionState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chip8ProcessorSnapshot {
    registers: Chip8ProcessorRegisters,
}

impl Component for Chip8Processor {
    fn query_components(&mut self, query: &QueryableComponents) {
        self.imported = Some(ImportedComponents {
            display: query.query_component("display").unwrap(),
            timer: query.query_component("timer").unwrap(),
            audio: query.query_component("audio").unwrap(),
        })
    }
}

impl SnapshotableComponent for Chip8Processor {
    fn save_snapshot(&mut self) -> rmpv::Value {
        todo!()
    }

    fn load_snapshot(&mut self, _state: rmpv::Value) {
        todo!()
    }
}

impl FromConfig for Chip8Processor {
    type Config = Chip8ProcessorConfig;

    fn from_config(_rom_manager: Arc<RomManager>, config: Self::Config) -> Self
    where
        Self: Sized,
    {
        Self {
            config,
            stack: ArrayVec::default(),
            registers: Chip8ProcessorRegisters::default(),
            imported: None,
            controller: None,
            execution_state: ExecutionState::Normal,
        }
    }
}

impl SchedulableComponent for Chip8Processor {
    fn tick_rate(&self) -> Ratio<u32> {
        self.config.frequency
    }

    fn tick(&mut self, _: &MemoryTranslationTable) {
        // The CPU is awaiting a key release
        match self.execution_state {
            ExecutionState::AwaitingKeyPress { register } => {
                if let Some(key) =
                    self.controller
                        .as_ref()
                        .unwrap()
                        .iter_pressed()
                        .find_map(|input| {
                            if let Ok(key) = Chip8Key::try_from(input) {
                                Some(key)
                            } else {
                                None
                            }
                        })
                {
                    self.execution_state = ExecutionState::AwaitingKeyRelease { register, key };
                }
            }
            ExecutionState::AwaitingKeyRelease { register, key } => {
                if !self
                    .controller
                    .as_ref()
                    .unwrap()
                    .get_input_state(key.try_into().unwrap())
                    .unwrap()
                    .as_digital()
                {
                    self.registers.work_registers[register as usize] = key.0;
                    self.execution_state = ExecutionState::Normal;
                }
            }
            _ => {}
        }
    }
}

impl ProcessorComponent for Chip8Processor {
    type InstructionSet = Chip8InstructionSet;

    // Chip8 has no timing concerns
    fn should_execution_occur(&self) -> bool {
        self.execution_state == ExecutionState::Normal
    }

    fn decompile(
        &self,
        cursor: usize,
        memory_translation_table: &MemoryTranslationTable,
    ) -> Result<(Self::InstructionSet, u8), InstructionDecompilingError>
    where
        Self: Sized,
    {
        let mut instruction = [0; 2];
        memory_translation_table
            .read(cursor, &mut instruction)
            .unwrap();

        let decompiled_instruction = decode_instruction(instruction).unwrap();

        Ok((decompiled_instruction, 2))
    }

    fn interpret(
        &mut self,
        program_pointer: &mut usize,
        instruction: Self::InstructionSet,
        memory_translation_table: &MemoryTranslationTable,
    ) -> Result<(), String> {
        // Delegated here because its really large
        self.interpret_instruction(program_pointer, instruction, memory_translation_table);

        Ok(())
    }
}

impl InputComponent for Chip8Processor {
    fn registered_inputs(&self) -> &'static [Input] {
        &[
            Input::Keyboard(KeyboardInput::Numpad1),
            Input::Keyboard(KeyboardInput::Numpad2),
            Input::Keyboard(KeyboardInput::Numpad3),
            Input::Keyboard(KeyboardInput::KeyC),
            Input::Keyboard(KeyboardInput::Numpad4),
            Input::Keyboard(KeyboardInput::Numpad5),
            Input::Keyboard(KeyboardInput::Numpad6),
            Input::Keyboard(KeyboardInput::KeyD),
            Input::Keyboard(KeyboardInput::Numpad7),
            Input::Keyboard(KeyboardInput::Numpad8),
            Input::Keyboard(KeyboardInput::Numpad9),
            Input::Keyboard(KeyboardInput::KeyE),
            Input::Keyboard(KeyboardInput::KeyA),
            Input::Keyboard(KeyboardInput::Numpad0),
            Input::Keyboard(KeyboardInput::KeyB),
            Input::Keyboard(KeyboardInput::KeyF),
        ]
    }

    fn assign_controller(&mut self, controller: Arc<EmulatedGamepad>) {
        self.controller = Some(controller);
    }
}
