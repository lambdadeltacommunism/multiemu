#[cfg(desktop)]
pub mod desktop;
#[cfg(nintendo_3ds)]
pub mod nintendo_3ds;
pub mod timing;

mod software_egui_render;

use crate::{
    component::display::DisplayComponent,
    rom::{GameSystem, RomId},
};
use egui::FullOutput;
use std::sync::{Arc, Mutex};

#[cfg(desktop)]
pub use desktop::display::software::SoftwareRendering;
#[cfg(desktop)]
pub use desktop::launch_gui;

#[cfg(nintendo_3ds)]
pub use nintendo_3ds::display::software::SoftwareRendering;
#[cfg(nintendo_3ds)]
pub use nintendo_3ds::launch_gui;

pub trait RenderingBackend {
    /// Data needed for a component to initialize itself for rendering
    type ComponentInitializationData: 'static;
    /// Intermediate image buffer to be shared with the runtime, typically arc wrapped
    type ComponentDisplayBuffer;

    type RuntimeState: RenderingBackendState<RenderingBackend = Self>;
}

#[allow(clippy::large_enum_variant)]
pub enum RedrawKind<'a, R: RenderingBackend> {
    Machine(&'a [Arc<Mutex<dyn DisplayComponent<R>>>]),
    Egui {
        context: &'a egui::Context,
        full_output: FullOutput,
    },
}

pub trait RenderingBackendState: Sized {
    type RenderingBackend: RenderingBackend;

    fn surface_resized(&mut self);

    fn redraw(&mut self, kind: RedrawKind<Self::RenderingBackend>);

    fn initialize_components(
        &mut self,
        components: &[Arc<Mutex<dyn DisplayComponent<Self::RenderingBackend>>>],
    );
}

pub enum InitialGuiState {
    MainMenu,
    OpenGame {
        user_specified_roms: Vec<RomId>,
        game_system: GameSystem,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RuntimeState {
    Menu,
    PlayingGame,
}
