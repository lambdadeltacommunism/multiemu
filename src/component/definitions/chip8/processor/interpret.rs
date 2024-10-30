use super::{
    input::Chip8Key,
    instruction::{Chip8InstructionSet, InstructionSetChip8},
    Chip8Processor, ExecutionState,
};
use crate::{
    component::{
        definitions::chip8::{Chip8Kind, CHIP8_FONT},
        memory::MemoryTranslationTable,
        processor::ProcessorComponent,
    },
    input::Input,
};
use arrayvec::ArrayVec;
use bitvec::{
    field::BitField,
    prelude::{Lsb0, Msb0},
    view::BitView,
};
use nalgebra::Point2;
use rand::{thread_rng, Rng};
use ringbuffer::RingBuffer;

impl Chip8Processor {
    pub fn interpret_instruction(
        &mut self,
        program_pointer: &mut usize,
        instruction: <Chip8Processor as ProcessorComponent>::InstructionSet,
        memory_translation_table: &MemoryTranslationTable,
    ) {
        let imported_components = self.imported.as_ref().unwrap();

        match instruction {
            Chip8InstructionSet::Chip8(InstructionSetChip8::Sys { syscall }) => match syscall {
                0x0e0 => {
                    imported_components.display.lock().unwrap().clear_display();
                }
                0x0ee => {
                    if let Some(address) = self.stack.pop() {
                        *program_pointer = address as usize;
                    } else {
                        tracing::error!("Stack underflow");
                        *program_pointer = 0x200;
                    }
                }
                _ => {
                    tracing::warn!("Unknown syscall: {:#04x}", syscall);
                }
            },
            Chip8InstructionSet::Chip8(InstructionSetChip8::Jump { address }) => {
                *program_pointer = address as usize;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Call { address }) => {
                self.stack.push(*program_pointer as u16);
                *program_pointer = address as usize;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Ske {
                register,
                immediate,
            }) => {
                let register_value = self.registers.work_registers[register as usize];

                if register_value == immediate {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Skne {
                register,
                immediate,
            }) => {
                let register_value = self.registers.work_registers[register as usize];

                if register_value != immediate {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Skre {
                param_register_1,
                param_register_2,
            }) => {
                let param_register_1_value =
                    self.registers.work_registers[param_register_1 as usize];
                let param_register_2_value =
                    self.registers.work_registers[param_register_2 as usize];

                if param_register_1_value == param_register_2_value {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Load {
                register,
                immediate,
            }) => {
                self.registers.work_registers[register as usize] = immediate;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Add {
                register,
                immediate,
            }) => {
                let register_value = self.registers.work_registers[register as usize];

                self.registers.work_registers[register as usize] =
                    register_value.wrapping_add(immediate);
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Move {
                param_register_1,
                param_register_2,
            }) => {
                self.registers.work_registers[param_register_1 as usize] =
                    self.registers.work_registers[param_register_2 as usize];
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Or {
                destination,
                source,
            }) => {
                self.registers.work_registers[destination as usize] |=
                    self.registers.work_registers[source as usize];

                if self.config.kind == Chip8Kind::Chip8 {
                    self.registers.work_registers[0xf] = 0;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::And {
                destination,
                source,
            }) => {
                self.registers.work_registers[destination as usize] &=
                    self.registers.work_registers[source as usize];

                if self.config.kind == Chip8Kind::Chip8 {
                    self.registers.work_registers[0xf] = 0;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Xor {
                destination,
                source,
            }) => {
                self.registers.work_registers[destination as usize] ^=
                    self.registers.work_registers[source as usize];

                if self.config.kind == Chip8Kind::Chip8 {
                    self.registers.work_registers[0xf] = 0;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Addr {
                destination,
                source,
            }) => {
                let destination_value = self.registers.work_registers[destination as usize];
                let source_value = self.registers.work_registers[source as usize];

                let (new_value, carry) = destination_value.overflowing_add(source_value);

                self.registers.work_registers[destination as usize] = new_value;
                self.registers.work_registers[0xf] = carry as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Sub {
                destination,
                source,
            }) => {
                let destination_value = self.registers.work_registers[destination as usize];
                let source_value = self.registers.work_registers[source as usize];

                let (new_value, borrow) = destination_value.overflowing_sub(source_value);

                self.registers.work_registers[destination as usize] = new_value;
                self.registers.work_registers[0xf] = !borrow as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Shr { register, value }) => {
                let mut destination_value = self.registers.work_registers[register as usize];

                if self.config.kind == Chip8Kind::Chip8 {
                    destination_value = self.registers.work_registers[value as usize];
                }

                let overflow = destination_value.view_bits::<Lsb0>()[0];

                self.registers.work_registers[register as usize] = destination_value >> 1;
                self.registers.work_registers[0xf] = overflow as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Subn {
                destination,
                source,
            }) => {
                let destination_value = self.registers.work_registers[destination as usize];
                let source_value = self.registers.work_registers[source as usize];

                let (new_value, borrow) = source_value.overflowing_sub(destination_value);

                self.registers.work_registers[destination as usize] = new_value;
                self.registers.work_registers[0xf] = !borrow as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Shl { register, value }) => {
                let mut destination_value = self.registers.work_registers[register as usize];

                if self.config.kind == Chip8Kind::Chip8 {
                    destination_value = self.registers.work_registers[value as usize];
                }

                let overflow = destination_value.view_bits::<Msb0>()[0];

                self.registers.work_registers[register as usize] = destination_value << 1;
                self.registers.work_registers[0xf] = overflow as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Skrne {
                param_register_1,
                param_register_2,
            }) => {
                let param_register_1_value =
                    self.registers.work_registers[param_register_1 as usize];
                let param_register_2_value =
                    self.registers.work_registers[param_register_2 as usize];

                if param_register_1_value != param_register_2_value {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Loadi { value }) => {
                self.registers.index = value;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Jumpi { address }) => {
                let address = if self.config.kind == Chip8Kind::Chip8 {
                    address.wrapping_add(self.registers.work_registers[0x0] as u16)
                } else {
                    let register = address.view_bits::<Msb0>()[4..8].load::<u8>();

                    address.wrapping_add(self.registers.work_registers[register as usize] as u16)
                };

                *program_pointer = address as usize;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Rand {
                register,
                immediate,
            }) => {
                self.registers.work_registers[register as usize] =
                    thread_rng().gen::<u8>() & immediate;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Draw {
                coordinate_registers,
                height,
            }) => {
                let mut buffer =
                    ArrayVec::<_, 16>::from_iter(std::iter::repeat(0).take(height as usize));

                let mut cursor = 0;
                for buffer_section in buffer.chunks_mut(2) {
                    memory_translation_table
                        .read(self.registers.index as usize + cursor, buffer_section)
                        .unwrap();
                    cursor += buffer_section.len();
                }

                let actual_coords = Point2::new(
                    self.registers.work_registers[coordinate_registers.x as usize],
                    self.registers.work_registers[coordinate_registers.y as usize],
                );

                // Sets VF to 1 if any pixel turned off otherwise set on
                self.registers.work_registers[0xf] = imported_components
                    .display
                    .lock()
                    .unwrap()
                    .draw_sprite(actual_coords, &buffer)
                    as u8;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Skpr { key }) => {
                let key_value = if let Ok(key) =
                    Input::try_from(Chip8Key(self.registers.work_registers[key as usize]))
                {
                    self.controller
                        .as_ref()
                        .unwrap()
                        .get_input_state(key)
                        .map(|state| state.as_digital())
                        .unwrap_or(false)
                } else {
                    false
                };

                if key_value {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Skup { key }) => {
                let key_value = if let Ok(key) =
                    Input::try_from(Chip8Key(self.registers.work_registers[key as usize]))
                {
                    self.controller
                        .as_ref()
                        .unwrap()
                        .get_input_state(key)
                        .map(|state| state.as_digital())
                        .unwrap_or(false)
                } else {
                    false
                };

                if !key_value {
                    *program_pointer += 2;
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Moved { register }) => {
                let delay_timer_value = imported_components.timer.lock().unwrap().delay_timer;

                self.registers.work_registers[register as usize] = delay_timer_value;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Keyd { key: register }) => {
                self.execution_state = ExecutionState::AwaitingKeyPress { register };
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Loadd { register }) => {
                let register_value = self.registers.work_registers[register as usize];

                imported_components.timer.lock().unwrap().delay_timer = register_value;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Loads { register }) => {
                let register_value = self.registers.work_registers[register as usize];

                imported_components.audio.lock().unwrap().sound_timer = register_value;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Addi { register }) => {
                let register_value = self.registers.work_registers[register as usize];

                self.registers.index = self.registers.index.wrapping_add(register_value as u16);
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Font { register }) => {
                let register_value = self.registers.work_registers[register as usize];

                self.registers.index = register_value as u16 * CHIP8_FONT[0].len() as u16;
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Bcd { register }) => {
                let register_value = self.registers.work_registers[register as usize];

                let [hundreds, tens, ones] = bcd_encode(register_value);

                memory_translation_table
                    .write(
                        self.registers.index as usize,
                        std::slice::from_ref(&hundreds),
                    )
                    .unwrap();
                memory_translation_table
                    .write(
                        self.registers.index as usize + 1,
                        std::slice::from_ref(&tens),
                    )
                    .unwrap();
                memory_translation_table
                    .write(
                        self.registers.index as usize + 2,
                        std::slice::from_ref(&ones),
                    )
                    .unwrap();
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Save { count }) => {
                for i in 0..=count {
                    memory_translation_table
                        .write(
                            self.registers.index as usize + i as usize,
                            &self.registers.work_registers[i as usize..=i as usize],
                        )
                        .unwrap();
                }

                // Only the original chip8 modifies the index register for this operation
                if self.config.kind == Chip8Kind::Chip8 {
                    self.registers.index = self.registers.index.wrapping_add(count as u16 + 1);
                }
            }
            Chip8InstructionSet::Chip8(InstructionSetChip8::Restore { count }) => {
                for i in 0..=count {
                    memory_translation_table
                        .read(
                            self.registers.index as usize + i as usize,
                            &mut self.registers.work_registers[i as usize..=i as usize],
                        )
                        .unwrap();
                }

                // Only the original chip8 modifies the index register for this operation
                if self.config.kind == Chip8Kind::Chip8 {
                    self.registers.index = self.registers.index.wrapping_add(count as u16 + 1);
                }
            }
            Chip8InstructionSet::SuperChip8(chip8_instruction_set_super) => todo!(),
            Chip8InstructionSet::XoChip(chip8_instruction_set_xo) => todo!(),
        }
    }
}

#[inline]
fn bcd_encode(value: u8) -> [u8; 3] {
    let hundreds = value / 100;
    let tens = (value / 10) % 10;
    let ones = value % 10;

    [hundreds, tens, ones]
}
