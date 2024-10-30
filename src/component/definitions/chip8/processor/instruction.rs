use crate::component::processor::{InstructionSet, InstructionTextRepresentation};

use nalgebra::Point2;
use std::ops::Range;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("Invalid instruction: {0:?}")]
    InvalidInstruction([u8; 2]),
    #[error("Unknown nom error")]
    Nom,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl TryFrom<u8> for Register {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::V0),
            1 => Ok(Register::V1),
            2 => Ok(Register::V2),
            3 => Ok(Register::V3),
            4 => Ok(Register::V4),
            5 => Ok(Register::V5),
            6 => Ok(Register::V6),
            7 => Ok(Register::V7),
            8 => Ok(Register::V8),
            9 => Ok(Register::V9),
            10 => Ok(Register::VA),
            11 => Ok(Register::VB),
            12 => Ok(Register::VC),
            13 => Ok(Register::VD),
            14 => Ok(Register::VE),
            15 => Ok(Register::VF),
            _ => Err(()),
        }
    }
}

// https://github.com/craigthomas/Chip8Assembler
// TODO: These mnemonics are terrible

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionSetChip8 {
    Sys {
        syscall: u16,
    },
    Jump {
        address: u16,
    },
    /// Jump but it adds the program counter to the stack
    Call {
        address: u16,
    },
    Ske {
        register: Register,
        immediate: u8,
    },
    Skne {
        register: Register,
        immediate: u8,
    },
    Skre {
        param_register_1: Register,
        param_register_2: Register,
    },
    Load {
        register: Register,
        immediate: u8,
    },
    Add {
        register: Register,
        immediate: u8,
    },
    Move {
        param_register_1: Register,
        param_register_2: Register,
    },
    Or {
        destination: Register,
        source: Register,
    },
    And {
        destination: Register,
        source: Register,
    },
    Xor {
        destination: Register,
        source: Register,
    },
    Addr {
        destination: Register,
        source: Register,
    },
    Sub {
        destination: Register,
        source: Register,
    },
    Shr {
        register: Register,
        value: Register,
    },
    Subn {
        destination: Register,
        source: Register,
    },
    Shl {
        register: Register,
        value: Register,
    },
    Skrne {
        param_register_1: Register,
        param_register_2: Register,
    },
    Loadi {
        value: u16,
    },
    Jumpi {
        address: u16,
    },
    Rand {
        register: Register,
        immediate: u8,
    },
    Draw {
        coordinate_registers: Point2<Register>,
        height: u8,
    },
    Skpr {
        key: Register,
    },
    Skup {
        key: Register,
    },
    Moved {
        register: Register,
    },
    Keyd {
        key: Register,
    },
    Loadd {
        register: Register,
    },
    Loads {
        register: Register,
    },
    Addi {
        register: Register,
    },
    Font {
        register: Register,
    },
    Bcd {
        register: Register,
    },
    Save {
        count: u8,
    },
    Restore {
        count: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionSetSuperChip8 {
    Scrd { amount: u8 },
    Scrr,
    Scrl,
    Srpl { amount: u8 },
    Rrpl { amount: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionSetXoChip {
    Ssub { bounds: Range<Register> },
    Rsub { bounds: Range<Register> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Chip8InstructionSet {
    Chip8(InstructionSetChip8),
    SuperChip8(InstructionSetSuperChip8),
    XoChip(InstructionSetXoChip),
}

impl InstructionSet for Chip8InstructionSet {
    fn to_text_representation(&self) -> InstructionTextRepresentation {
        todo!()
    }
}
