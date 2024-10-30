use super::{memory::MemoryTranslationTable, schedulable::SchedulableComponent};
use std::fmt::Debug;
use std::{borrow::Cow, fmt::Display};
use thiserror::Error;

/// The result of compiling an instruction was not ok
#[derive(Error, Debug)]
pub enum InstructionDecompilingError {
    #[error("The instruction could not be decompiled: {0:x?}")]
    InstructionDecompilingFailed(Vec<u8>),
}

#[derive(Debug)]
pub struct InstructionTextRepresentation {
    pub instruction_mnemonic: Cow<'static, str>,
}

impl Display for InstructionTextRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction_mnemonic)
    }
}

pub trait InstructionSet: Debug + Sized {
    fn to_text_representation(&self) -> InstructionTextRepresentation;
}

pub trait ProcessorComponent: SchedulableComponent {
    type InstructionSet: InstructionSet;

    fn should_execution_occur(&self) -> bool;

    fn decompile(
        &self,
        cursor: usize,
        memory_translation_table: &MemoryTranslationTable,
    ) -> Result<(Self::InstructionSet, u8), InstructionDecompilingError>;

    fn interpret(
        &mut self,
        program_pointer: &mut usize,
        instruction: Self::InstructionSet,
        memory_translation_table: &MemoryTranslationTable,
    ) -> Result<(), String>;
}
