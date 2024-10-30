use super::Component;
use arrayvec::ArrayVec;
use std::{
    ops::Range,
    sync::{Arc, Mutex},
};
use thiserror::Error;

pub trait MemoryComponent: Component {
    fn assigned_memory_range(&self) -> Range<usize>;

    fn read_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, ReadMemoryRecord), 8>,
    ) -> u64;

    fn write_memory(
        &mut self,
        address: usize,
        buffer: &[u8],
        records: &mut ArrayVec<(Range<usize>, WriteMemoryRecord), 8>,
    ) -> u64;

    // Its like read_memory but without the restriction on the size of the buffer and it cannot cause a state change
    fn preview_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
        records: &mut ArrayVec<(Range<usize>, PreviewMemoryRecord), 8>,
    );
}

pub fn relocate_and_crop_range(from: &Range<usize>, to: &Range<usize>) -> Range<usize> {
    let from_start = from.start as i128;
    let from_end = from.end as i128;
    let to_start = to.start as i128;
    let to_end = to.end as i128;

    // Calculate the offset between from and to
    let offset = from_start - to_start;

    // Adjust the start and end of the from range according to the offset
    let relocated_start = from_start - offset;
    let relocated_end = from_end - offset;

    // Ensure the relocated range is within the bounds of to
    let start = relocated_start.max(to_start);
    let end = relocated_end.min(to_end);

    // Return the resulting range as usize
    start as usize..end as usize
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadMemoryRecord {
    /// Memory could not be read
    Denied,
    /// Memory redirects somewhere else
    Redirect { offset: usize },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WriteMemoryRecord {
    /// Memory could not be written
    Denied,
    /// Memory redirects somewhere else
    Redirect { offset: usize },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreviewMemoryRecord {
    /// Memory denied
    Denied,
    /// Memory redirects somewhere else
    Redirect {
        offset: usize,
    },
    // Memory here can't be read without an intense calculation or a state change
    PreviewImpossible,
}

#[derive(Error, Debug)]
pub enum MemoryOperationError {
    #[error("Memory could not be read/written/previewed")]
    Denied(Range<usize>),
    #[error("Memory access is out of bounds")]
    OutOfBounds(Range<usize>),
}

#[derive(Default)]
pub struct MemoryTranslationTable {
    entries: Vec<(Range<usize>, Arc<Mutex<dyn MemoryComponent>>)>,
}

impl MemoryTranslationTable {
    pub fn insert(&mut self, range: Range<usize>, component: Arc<Mutex<dyn MemoryComponent>>) {
        self.entries.push((range, component));
    }

    /// Get the component at a given address
    pub fn get(&self, address: usize) -> Option<Arc<Mutex<dyn MemoryComponent>>> {
        self.entries
            .iter()
            .find(|(range, _)| range.contains(&address))
            .map(|(_, component)| component.clone())
    }

    /// Check if an entry is overlapped
    pub fn is_overlapped(&self, new_range: Range<usize>) -> bool {
        self.entries.iter().any(|(existing_range, _)| {
            existing_range.start < new_range.end && new_range.start < existing_range.end
        })
    }

    /// Get all components that overlap with a range with their overlapping portions
    pub fn overlaps(
        &self,
        target: Range<usize>,
    ) -> impl Iterator<Item = (Range<usize>, &Arc<Mutex<dyn MemoryComponent>>)> + '_ {
        self.entries.iter().filter_map(move |(range, component)| {
            // Check if there is an overlap
            if range.start < target.end && range.end > target.start {
                // Crop range to the overlapping portion
                let overlap_start = range.start.max(target.start);
                let overlap_end = range.end.min(target.end);

                // Only return non-zero-length ranges
                if overlap_start < overlap_end {
                    let cropped_range = overlap_start..overlap_end;
                    return Some((cropped_range, component));
                }
            }
            None
        })
    }

    #[inline]
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> Result<u64, MemoryOperationError> {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));

        // Calculate the actual range that the buffer will be reading from
        let buffer_target_range = offset..offset + buffer.len();
        let mut cycles = 0;
        let mut to_inspect = ArrayVec::<_, 8>::default();

        to_inspect.extend(self.overlaps(buffer_target_range.clone()));

        if to_inspect.is_empty() {
            return Err(MemoryOperationError::OutOfBounds(buffer_target_range));
        }

        while let Some((entry_range, memory_component)) = to_inspect.pop() {
            let buffer_subsection = relocate_and_crop_range(&entry_range, &(0..buffer.len()));
            let mut records = ArrayVec::default();

            let mut memory_component = memory_component.lock().unwrap();
            let cycles_taken = memory_component.read_memory(
                entry_range.start,
                &mut buffer[buffer_subsection],
                &mut records,
            );
            cycles += cycles_taken;

            for (context_range, error) in records {
                match error {
                    ReadMemoryRecord::Denied => {
                        return Err(MemoryOperationError::Denied(context_range));
                    }
                    ReadMemoryRecord::Redirect { offset } => {
                        let context_range =
                            relocate_and_crop_range(&context_range, &(0..buffer.len()));
                        let context_range =
                            context_range.start + offset..context_range.end + offset;
                        to_inspect.extend(self.overlaps(context_range));
                    }
                }
            }
        }

        Ok(cycles)
    }

    #[inline]
    pub fn write(&self, offset: usize, buffer: &[u8]) -> Result<u64, MemoryOperationError> {
        debug_assert!([1, 2, 4, 8].contains(&buffer.len()));

        // Calculate the actual range that the buffer will be reading from
        let buffer_target_range = offset..offset + buffer.len();
        let mut cycles = 0;
        let mut to_inspect =
            ArrayVec::<_, 8>::from_iter(self.overlaps(buffer_target_range.clone()));

        if to_inspect.is_empty() {
            return Err(MemoryOperationError::OutOfBounds(buffer_target_range));
        }

        while let Some((entry_range, memory_component)) = to_inspect.pop() {
            let buffer_subsection = relocate_and_crop_range(&entry_range, &(0..buffer.len()));
            let mut records = ArrayVec::default();

            let mut memory_component = memory_component.lock().unwrap();
            let cycles_taken = memory_component.write_memory(
                entry_range.start,
                &buffer[buffer_subsection],
                &mut records,
            );
            cycles += cycles_taken;

            for (context_range, error) in records {
                match error {
                    WriteMemoryRecord::Denied => {
                        return Err(MemoryOperationError::Denied(context_range));
                    }
                    WriteMemoryRecord::Redirect { offset } => {
                        let context_range =
                            relocate_and_crop_range(&context_range, &(0..buffer.len()));
                        let context_range =
                            context_range.start + offset..context_range.end + offset;
                        to_inspect.extend(self.overlaps(context_range));
                    }
                }
            }
        }

        Ok(cycles)
    }

    pub fn preview(&self, offset: usize, buffer: &mut [u8]) -> Result<(), MemoryOperationError> {
        // Calculate the actual range that the buffer will be reading from
        let buffer_target_range = offset..offset + buffer.len();
        // We use a vec here cuz buffer could be infinitely large
        let mut to_inspect = Vec::new();

        to_inspect.extend(self.overlaps(buffer_target_range.clone()));

        if to_inspect.is_empty() {
            return Err(MemoryOperationError::OutOfBounds(buffer_target_range));
        }

        while let Some((entry_range, memory_component)) = to_inspect.pop() {
            let buffer_subsection = relocate_and_crop_range(&entry_range, &(0..buffer.len()));
            let mut records = ArrayVec::default();

            let mut memory_component = memory_component.lock().unwrap();
            memory_component.preview_memory(
                entry_range.start,
                &mut buffer[buffer_subsection],
                &mut records,
            );

            for (context_range, error) in records {
                match error {
                    PreviewMemoryRecord::Denied => {
                        return Err(MemoryOperationError::Denied(context_range));
                    }
                    PreviewMemoryRecord::Redirect { offset } => {
                        let context_range =
                            relocate_and_crop_range(&context_range, &(0..buffer.len()));
                        let context_range =
                            context_range.start + offset..context_range.end + offset;
                        to_inspect.extend(self.overlaps(context_range));
                    }
                    PreviewMemoryRecord::PreviewImpossible => {
                        todo!()
                    }
                }
            }
        }

        Ok(())
    }
}

pub enum MemoryPermission {
    Read,
    Write,
    Execute,
}
