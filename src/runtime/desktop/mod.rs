use super::{
    timing::FramerateTracker, InitialGuiState, RedrawKind, RenderingBackend, RenderingBackendState,
};
use crate::{
    component::{definitions::chip8::display::Chip8Display, display::DisplayComponent},
    config::GlobalConfig,
    gui::{GuiRuntime, UiOutput},
    input::InputState,
    machine::{
        definitions::construct_machine,
        executor::{single::SingleThreadedExecutor, Executor},
    },
    rom::{GameSystem, RomId, RomManager},
};
use display::WinitRenderBackendState;
use egui::ViewportId;
use egui_winit::EventResponse;
use gamepad::GilrsGamepadManager;
use std::sync::{Arc, Mutex, RwLock};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

pub mod audio;
pub mod display;
pub mod gamepad;

/// Tracks if we are running or should be running a game
enum MachineContextState<E: Executor, R: RenderingBackend> {
    /// Machine is waiting for graphics context to be ready
    Pending {
        user_specified_roms: Vec<RomId>,
        forced_system: Option<GameSystem>,
    },
    /// Machine is currently running
    Running {
        machine_context: MachineContext<E, R>,
    },
}

struct WindowingContext<R: RenderingBackend> {
    /// Winit window handle
    window: Arc<Window>,
    /// The rendering backend in question
    display_backend_state: R::RuntimeState,
    /// Winit specific egui context
    egui_winit_context: egui_winit::State,
}

/// Stuff needed for a running emulation
struct MachineContext<E: Executor, R: RenderingBackend> {
    executor: E,
    /// Intermediate buffer components render to
    display_components: Vec<Arc<Mutex<dyn DisplayComponent<R>>>>,
    /// gamepad translation table
    gamepad_manager: GilrsGamepadManager,
}

pub struct DesktopRuntime<E: Executor, R: RenderingBackend> {
    /// Tracks frame durations so we can execute at a regular interval
    framerate_tracker: FramerateTracker,
    /// Egui context
    egui_context: egui::Context,
    /// Ui state
    gui_state: GuiRuntime,
    /// Late initialized data for working with the windowing system
    windowing_context: Option<WindowingContext<R>>,
    /// The game that's currently running
    machine_context_state: Option<MachineContextState<E, R>>,
    /// The system rom manager
    rom_manager: Arc<RomManager>,
    /// The global config
    global_config: Arc<RwLock<GlobalConfig>>,
}

impl<E: Executor, R: RenderingBackend> DesktopRuntime<E, R> {
    pub fn new(rom_manager: Arc<RomManager>, global_config: Arc<RwLock<GlobalConfig>>) -> Self {
        Self {
            framerate_tracker: FramerateTracker::default(),
            egui_context: egui::Context::default(),
            gui_state: GuiRuntime::new(global_config.clone()),
            windowing_context: None,
            machine_context_state: None,
            rom_manager,
            global_config,
        }
    }

    pub fn new_with_game(
        rom_manager: Arc<RomManager>,
        user_specified_roms: Vec<RomId>,
        forced_system: Option<GameSystem>,
        global_config: Arc<RwLock<GlobalConfig>>,
    ) -> Self {
        let mut me = Self::new(rom_manager, global_config);

        me.machine_context_state = Some(MachineContextState::Pending {
            user_specified_roms,
            forced_system,
        });

        me
    }

    pub fn setup_window(&mut self, event_loop: &ActiveEventLoop) -> Arc<Window> {
        let window_attributes = Window::default_attributes()
            .with_title("MultiEMU")
            .with_resizable(true)
            // TODO: Add a fullscreen knob on the global config
            .with_inner_size(PhysicalSize::new(640, 480));
        Arc::new(event_loop.create_window(window_attributes).unwrap())
    }

    pub fn is_gui_active(&self) -> bool {
        // This helps the user not stare at a black screen
        self.gui_state.active
            || !matches!(
                self.machine_context_state,
                Some(MachineContextState::Running { .. })
            )
    }
}

impl<E: Executor, R: RenderingBackend> ApplicationHandler for DesktopRuntime<E, R>
where
    R::RuntimeState: WinitRenderBackendState,
    Chip8Display: DisplayComponent<R>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // HACK: This will cause frequent crashes on mobile platforms
        if self.windowing_context.is_some() {
            panic!("Window already created");
        }

        let window = self.setup_window(event_loop);
        let mut rendering_state = R::RuntimeState::new(window.clone(), self.global_config.clone());
        // A hack that only works because this application only uses one window
        let viewport_id = ViewportId::ROOT;
        let egui_winit_context = egui_winit::State::new(
            self.egui_context.clone(),
            viewport_id,
            &window,
            None,
            None,
            None,
        );

        match self.machine_context_state.take() {
            Some(MachineContextState::Pending {
                user_specified_roms,
                forced_system,
            }) => {
                // FIXME: In no way is this sound. Roms can very much have disagreeing systems
                let game_system = forced_system.unwrap_or_else(|| {
                    self.rom_manager.rom_information[&user_specified_roms[0]].system
                });

                let machine = construct_machine::<R>(
                    game_system,
                    self.rom_manager.clone(),
                    user_specified_roms,
                    &mut rendering_state,
                );

                let executor = E::new(machine.tasks, machine.memory_translation_table.clone());

                self.gui_state.active = false;
                self.machine_context_state = Some(MachineContextState::Running {
                    machine_context: MachineContext {
                        executor,
                        display_components: machine.display_components,
                        gamepad_manager: GilrsGamepadManager::new(
                            machine.controllers,
                            game_system,
                            self.global_config.clone(),
                        ),
                    },
                });
            }
            Some(MachineContextState::Running { .. }) => {
                panic!("Windowing was initialized while a machine was active somehow");
            }
            None => {}
        }

        self.windowing_context = Some(WindowingContext {
            window,
            display_backend_state: rendering_state,
            egui_winit_context,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // This helps the user not stare at a black screen
        let is_gui_active = self.is_gui_active();

        let window_context = self
            .windowing_context
            .as_mut()
            .expect("Window was not initialized");

        // Ensure a resize happens before drawing occurs
        if matches!(event, WindowEvent::Resized(_)) {
            window_context.display_backend_state.surface_resized();
            return;
        }

        if is_gui_active || matches!(event, WindowEvent::ScaleFactorChanged { .. }) {
            let EventResponse {
                consumed,
                repaint: _,
            } = window_context
                .egui_winit_context
                .on_window_event(&window_context.window, &event);

            if consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if is_synthetic {
                    return;
                }

                if !is_gui_active {
                    let Some(MachineContextState::Running { machine_context }) =
                        self.machine_context_state.as_mut()
                    else {
                        tracing::error!("Menu is not active yet no machine is running, the user is probably sitting at a black screen right now");
                        return;
                    };

                    let PhysicalKey::Code(key) = event.physical_key else {
                        return;
                    };

                    machine_context.gamepad_manager.insert_input(
                        key.try_into().unwrap(),
                        InputState::Digital(event.state == ElementState::Pressed),
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                if is_gui_active {
                    // Grabbing the ui output is a little unpleasant here
                    let mut ui_output = None;
                    let full_output = self.egui_context.run(
                        window_context
                            .egui_winit_context
                            .take_egui_input(&window_context.window),
                        |context| {
                            ui_output = ui_output.take().or(self.gui_state.run_menu(context));
                        },
                    );

                    match ui_output {
                        Some(UiOutput::OpenGame { path }) => {
                            tracing::info!("Opening {} by order of the gui", path.display());
                        }
                        None => {}
                    }

                    window_context
                        .display_backend_state
                        .redraw(RedrawKind::Egui {
                            context: &self.egui_context,
                            full_output,
                        });
                } else {
                    let Some(MachineContextState::Running { machine_context }) =
                        self.machine_context_state.as_mut()
                    else {
                        tracing::error!("Menu is not active yet no machine is running, the user is probably sitting at a black screen right now");
                        return;
                    };
                    self.framerate_tracker.record_frame();
                    window_context
                        .display_backend_state
                        .redraw(RedrawKind::Machine(&machine_context.display_components));
                    machine_context
                        .executor
                        .run(self.framerate_tracker.average_framerate());
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.windowing_context
            .as_mut()
            .unwrap()
            .window
            .request_redraw();
    }
}

impl<E: Executor, R: RenderingBackend> Drop for DesktopRuntime<E, R> {
    fn drop(&mut self) {
        // Prevents a segfault
        self.windowing_context = None;
    }
}

pub fn launch_gui<R: RenderingBackend>(
    rom_manager: Arc<RomManager>,
    initial_gui_state: InitialGuiState,
    global_config: Arc<RwLock<GlobalConfig>>,
) where
    DesktopRuntime<SingleThreadedExecutor, R>: ApplicationHandler,
    // TODO: find some better way to express these bounds
    Chip8Display: DisplayComponent<R>,
{
    let mut winit_state = match initial_gui_state {
        InitialGuiState::MainMenu => {
            DesktopRuntime::<SingleThreadedExecutor, R>::new(rom_manager, global_config)
        }
        InitialGuiState::OpenGame {
            user_specified_roms,
            game_system,
        } => DesktopRuntime::<SingleThreadedExecutor, R>::new_with_game(
            rom_manager,
            user_specified_roms,
            Some(game_system),
            global_config,
        ),
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut winit_state).unwrap();
}
