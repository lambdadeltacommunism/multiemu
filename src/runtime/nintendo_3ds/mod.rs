use super::{InitialGuiState, RenderingBackend, RenderingBackendState};
use crate::{
    component::{definitions::chip8::display::Chip8Display, display::DisplayComponent},
    config::GlobalConfig,
    gui::GuiRuntime,
    machine::executor::{single::SingleThreadedExecutor, Executor},
    rom::RomManager,
};
use ctru::{
    prelude::{Apt, Console, Gfx},
    services::gfx::{Flush, Screen, Swap},
};
use display::Nintendo3dsRenderBackendState;
use egui::{FullOutput, RawInput};
use nalgebra::Vector2;
use std::{
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

pub mod display;

/// Stuff needed for a running emulation
struct MachineContext<E: Executor, R: RenderingBackend> {
    executor: E,
    /// Intermediate buffer components render to
    display_components: Vec<Arc<Mutex<dyn DisplayComponent<R>>>>,
}

pub struct Nintendo3dsRuntime<E: Executor, R: RenderingBackend> {
    applet_service: Apt,
    graphics_service: Rc<Gfx>,
    machine_context: Option<MachineContext<E, R>>,
    egui_context: egui::Context,
    gui_state: GuiRuntime,
    display_runtime_state: R::RuntimeState,
}

impl<E: Executor, R: RenderingBackend> Nintendo3dsRuntime<E, R>
where
    R::RuntimeState: Nintendo3dsRenderBackendState,
{
    pub fn new(global_config: Arc<RwLock<GlobalConfig>>) -> Self {
        let apt = Apt::new().unwrap();

        let (display_runtime_state, gfx) = R::RuntimeState::new();

        let egui_context = egui::Context::default();

        Self {
            applet_service: apt,
            graphics_service: gfx,
            machine_context: None,
            gui_state: GuiRuntime::new(global_config.clone()),
            egui_context,
            display_runtime_state,
        }
    }

    pub fn run(&mut self) {
        while self.applet_service.main_loop() {
            let mut screen_framebuffer_guard = self.graphics_service.top_screen.borrow_mut();
            let screen_framebuffer = screen_framebuffer_guard.raw_framebuffer();
            let screen_dimensions =
                Vector2::new(screen_framebuffer.height, screen_framebuffer.width).cast();
            drop(screen_framebuffer_guard);

            let input = RawInput {
                screen_rect: Some(egui::Rect::from_min_max(
                    (0.0, 0.0).into(),
                    (screen_dimensions.x, screen_dimensions.y).into(),
                )),
                ..Default::default()
            };

            let full_output = self.egui_context.run(input, |context| {
                self.gui_state.main_menu_logic(context);
            });

            //console.flush_buffers();
            self.display_runtime_state
                .redraw_egui(&self.egui_context, full_output);
            self.graphics_service.wait_for_vblank();
        }
    }
}

pub fn launch_gui<R: RenderingBackend>(
    rom_manager: Arc<RomManager>,
    initial_gui_state: InitialGuiState,
    global_config: Arc<RwLock<GlobalConfig>>,
) where
    // TODO: find some better way to express these bounds
    Chip8Display: DisplayComponent<R>,
    R::RuntimeState: Nintendo3dsRenderBackendState,
{
    let mut runtime = Nintendo3dsRuntime::<SingleThreadedExecutor, R>::new(global_config);
    runtime.run();
}
