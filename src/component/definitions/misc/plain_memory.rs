use crate::{
    component::{
        memory::{MemoryComponent, PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord},
        snapshot::SnapshotableComponent,
        Component, FromConfig,
    },
    rom::{RomId, RomManager, RomRequirement},
};
use arrayvec::ArrayVec;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::{io::Read, ops::Range, sync::Arc};

#[derive(Debug)]
pub enum PlainMemoryInitialContents {
    Value { value: u8 },
    Array { value: &'static [u8], offset: usize },
    Rom { rom_id: RomId, offset: usize },
    Random,
}

#[derive(Debug)]
pub struct PlainMemoryConfig {
    // If the buffer is readable
    pub readable: bool,
    // If the buffer is writable
    pub writable: bool,
    // The maximum word size
    pub max_word_size: u8,
    // The penalty for each cycle
    pub read_cycle_penalty_calculator: fn(range: Range<usize>, denied: bool) -> u64,
    pub write_cycle_penalty_calculator: fn(range: Range<usize>, denied: bool) -> u64,
    // Memory region this buffer will be mapped to
    pub assigned_range: Range<usize>,
    // Initial contents
    pub initial_contents: PlainMemoryInitialContents,
}

impl Default for PlainMemoryConfig {
    fn default() -> Self {
        Self {
            readable: true,
            writable: true,
            max_word_size: 8,
            read_cycle_penalty_calculator: |_, _| 0,
            write_cycle_penalty_calculator: |_, _| 0,
            assigned_range: 0..0,
            initial_contents: PlainMemoryInitialContents::Value { value: 0 },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlainMemorySnapshot {
    pub memory: Vec<u8>,
}

pub struct PlainMemory {
    config: PlainMemoryConfig,
    rom_manager: Arc<RomManager>,
    buffer: Vec<u8>,
}

impl Component for PlainMemory {
    fn reset(&mut self) {
        initialize_internal_buffer(&self.config, &mut self.buffer, &self.rom_manager);
    }
}

impl SnapshotableComponent for PlainMemory {
    fn save_snapshot(&mut self) -> rmpv::Value {
        let state = PlainMemorySnapshot {
            memory: self.buffer.clone(),
        };

        rmpv::ext::to_value(&state).unwrap()
    }

    fn load_snapshot(&mut self, state: rmpv::Value) {
        let state = rmpv::ext::from_value::<PlainMemorySnapshot>(state).unwrap();

        // This also does size validation
        self.buffer.copy_from_slice(&state.memory);
    }
}

impl FromConfig for PlainMemory {
    type Config = PlainMemoryConfig;

    fn from_config(rom_manager: Arc<RomManager>, config: Self::Config) -> Self {
        assert!(
            [1, 2, 4, 8].contains(&config.max_word_size),
            "Invalid word size"
        );
        assert!(
            !config.assigned_range.is_empty(),
            "Memory assigned must be non-empty"
        );

        let buffer_size = config.assigned_range.clone().count();

        let mut buffer = vec![0; buffer_size];

        initialize_internal_buffer(&config, &mut buffer, &rom_manager);

        Self {
            config,
            buffer,
            rom_manager,
        }
    }
}

fn initialize_internal_buffer(
    config: &PlainMemoryConfig,
    buffer: &mut [u8],
    rom_manager: &RomManager,
) {
    match config.initial_contents {
        PlainMemoryInitialContents::Value { value } => {
            buffer.fill(value);
        }
        PlainMemoryInitialContents::Random => {
            thread_rng().fill_bytes(buffer);
        }
        PlainMemoryInitialContents::Array {
            value: data,
            offset,
        } => {
            let adjusted_offset = offset - config.assigned_range.start;
            buffer[adjusted_offset..adjusted_offset + data.len()].copy_from_slice(data);
        }
        PlainMemoryInitialContents::Rom { rom_id, offset } => {
            let mut rom_buffer = Vec::new();

            let mut rom_file = rom_manager.open(rom_id, RomRequirement::Required).unwrap();
            rom_file.read_to_end(&mut rom_buffer).unwrap();

            let adjusted_offset = offset - config.assigned_range.start;
            buffer[adjusted_offset..adjusted_offset + rom_buffer.len()]
                .copy_from_slice(&rom_buffer);
        }
    }
}

impl MemoryComponent for PlainMemory {
    fn assigned_memory_range(&self) -> Range<usize> {
        self.config.assigned_range.clone()
    }

    fn read_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, ReadMemoryRecord), 8>,
    ) -> u64 {
        debug_assert!(
            [1, 2, 4, 8].contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );

        let affected_range = address..address + buffer.len();

        if !self.config.readable {
            records.push((affected_range.clone(), ReadMemoryRecord::Denied));

            return (self.config.read_cycle_penalty_calculator)(affected_range, true);
        }

        if buffer.len() > self.config.max_word_size as usize {
            records.push((affected_range.clone(), ReadMemoryRecord::Denied));

            return (self.config.read_cycle_penalty_calculator)(affected_range, true);
        }

        let address_range = address - self.config.assigned_range.start
            ..address + buffer.len() - self.config.assigned_range.start;

        buffer.copy_from_slice(&self.buffer[address_range.clone()]);

        (self.config.read_cycle_penalty_calculator)(affected_range, false)
    }

    fn write_memory(
        &mut self,
        address: usize,
        buffer: &[u8],
        records: &mut ArrayVec<(Range<usize>, WriteMemoryRecord), 8>,
    ) -> u64 {
        debug_assert!(
            [1, 2, 4, 8].contains(&buffer.len()),
            "Invalid memory access size {}",
            buffer.len()
        );

        let affected_range = address..address + buffer.len();

        if !self.config.writable {
            records.push((affected_range.clone(), WriteMemoryRecord::Denied));

            return (self.config.write_cycle_penalty_calculator)(affected_range, true);
        }

        if buffer.len() > self.config.max_word_size as usize {
            records.push((affected_range.clone(), WriteMemoryRecord::Denied));

            return (self.config.write_cycle_penalty_calculator)(affected_range, true);
        }

        let address_range = address - self.config.assigned_range.start
            ..address + buffer.len() - self.config.assigned_range.start;

        self.buffer[address_range.clone()].copy_from_slice(buffer);

        (self.config.write_cycle_penalty_calculator)(affected_range, false)
    }

    fn preview_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, PreviewMemoryRecord), 8>,
    ) {
        let affected_range = address..address + buffer.len();

        if !self.config.readable {
            records.push((affected_range.clone(), PreviewMemoryRecord::Denied));
            return;
        }

        let address_range = address - self.config.assigned_range.start
            ..address + buffer.len() - self.config.assigned_range.start;

        buffer.copy_from_slice(&self.buffer[address_range]);
    }
}
