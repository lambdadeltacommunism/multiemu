use crate::{
    component::{
        memory::{MemoryComponent, PreviewMemoryRecord, ReadMemoryRecord, WriteMemoryRecord},
        Component, FromConfig,
    },
    rom::{RomId, RomManager, RomRequirement},
};
use arrayvec::ArrayVec;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    ops::Range,
    sync::Arc,
};

#[derive(Debug)]
pub struct RomMemoryConfig {
    pub rom_id: RomId,
    // The maximum word size
    pub max_word_size: u8,
    // The penalty for each cycle
    pub read_cycle_penalty_calculator: fn(range: Range<usize>, denied: bool) -> u64,
    pub write_cycle_penalty_calculator: fn(range: Range<usize>) -> u64,
    // Memory region this buffer will be mapped to
    pub assigned_range: Range<usize>,
}

impl Default for RomMemoryConfig {
    fn default() -> Self {
        Self {
            rom_id: RomId::new([0; 20]),
            max_word_size: 8,
            read_cycle_penalty_calculator: |_, _| 0,
            write_cycle_penalty_calculator: |_| 0,
            assigned_range: 0..0,
        }
    }
}

pub struct RomMemory {
    config: RomMemoryConfig,
    rom: BufReader<File>,
}

impl Component for RomMemory {}

impl FromConfig for RomMemory {
    type Config = RomMemoryConfig;

    fn from_config(rom_manager: Arc<RomManager>, config: Self::Config) -> Self
    where
        Self: Sized,
    {
        let rom_file = rom_manager
            .open(config.rom_id, RomRequirement::Required)
            .unwrap();

        Self {
            config,
            rom: BufReader::new(rom_file),
        }
    }
}

impl MemoryComponent for RomMemory {
    fn assigned_memory_range(&self) -> Range<usize> {
        self.config.assigned_range.clone()
    }

    fn read_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, ReadMemoryRecord), 8>,
    ) -> u64 {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));

        let affected_range = address..address + buffer.len();

        if buffer.len() > self.config.max_word_size as usize {
            records.push((affected_range.clone(), ReadMemoryRecord::Denied));

            return (self.config.read_cycle_penalty_calculator)(affected_range, true);
        }

        let address = self.get_relative_address(address);
        self.rom.seek(SeekFrom::Start(address as u64)).unwrap();
        self.rom.read_exact(buffer).unwrap();

        (self.config.read_cycle_penalty_calculator)(affected_range, false)
    }

    fn write_memory(
        &mut self,
        address: usize,
        buffer: &[u8],
        records: &mut ArrayVec<(Range<usize>, WriteMemoryRecord), 8>,
    ) -> u64 {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));
        records.push((address..address + buffer.len(), WriteMemoryRecord::Denied));

        (self.config.write_cycle_penalty_calculator)(address..address + buffer.len())
    }

    fn preview_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        _records: &mut ArrayVec<(Range<usize>, PreviewMemoryRecord), 8>,
    ) {
        let address = self.get_relative_address(address);
        self.rom.seek(SeekFrom::Start(address as u64)).unwrap();
        self.rom.read_exact(buffer).unwrap();
    }
}

impl RomMemory {
    fn get_relative_address(&self, address: usize) -> usize {
        address - self.config.assigned_range.start
    }
}
