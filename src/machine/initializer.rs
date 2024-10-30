use super::Machine;
use crate::{
    rom::{RomId, RomManager},
    runtime::RenderingBackend,
};
use sealed::sealed;
use std::sync::Arc;

#[sealed]
pub trait MachineInitializer<R: RenderingBackend> {
    fn initialize(
        &mut self,
        rom_manager: Arc<RomManager>,
        user_specified_roms: Vec<RomId>,
        rendering_state: &mut <R as RenderingBackend>::RuntimeState,
    ) -> Machine<R>;
}

#[sealed]
impl<
        R: RenderingBackend,
        F: FnMut(
            Arc<RomManager>,
            Vec<RomId>,
            &mut <R as RenderingBackend>::RuntimeState,
        ) -> Machine<R>,
    > MachineInitializer<R> for F
{
    fn initialize(
        &mut self,
        rom_manager: Arc<RomManager>,
        user_specified_roms: Vec<RomId>,
        rendering_state: &mut <R as RenderingBackend>::RuntimeState,
    ) -> Machine<R> {
        self(rom_manager, user_specified_roms, rendering_state)
    }
}
