// Required for audio support
#![cfg_attr(nintendo_3ds, feature(allocator_api))]

use config::GlobalConfig;
use env::{IMPORTED_ROM_DIRECTORY, LOG_LOCATION, ROM_DATABASE_PATH, STORAGE_DIRECTORY};
use rom::RomManager;
use runtime::{launch_gui, InitialGuiState};
use std::{
    error::Error,
    fs::{create_dir_all, File},
    ops::Deref,
    sync::{Arc, RwLock},
};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use runtime::SoftwareRendering;

#[cfg(desktop)]
mod cli;
mod component;
mod config;
mod env;
mod gui;
mod input;
mod machine;
mod rom;
mod runtime;
mod snapshot;
mod task;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(nintendo_3ds)]
    ctru::applets::error::set_panic_hook(true);

    let _ = create_dir_all(STORAGE_DIRECTORY.deref());
    let log_file = File::create(LOG_LOCATION.deref())?;
    let (log_writer, _log_writer_guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .with_writer(log_writer)
        .with_ansi(false)
        .init();

    tracing::info!("MultiEMU v{}", env!("CARGO_PKG_VERSION"));

    let mut global_config = GlobalConfig::default();
    let _ = global_config.load();
    let global_config = Arc::new(RwLock::new(global_config));

    #[cfg(desktop)]
    {
        use clap::Parser;
        use cli::handle_cli;
        use cli::Cli;

        let cli_arguments = Cli::parse();

        if let Some(action) = cli_arguments.action {
            handle_cli(action, global_config.clone());

            global_config.read().unwrap().save()?;

            return Ok(());
        }
    }

    let mut rom_manager = RomManager::default();

    create_dir_all(IMPORTED_ROM_DIRECTORY.deref())?;
    let _ = rom_manager.load_rom_paths(IMPORTED_ROM_DIRECTORY.deref());
    let _ = rom_manager.load_rom_info(ROM_DATABASE_PATH.deref());
    let rom_manager = Arc::new(rom_manager);

    if global_config.read().unwrap().hardware_acceleration {
        #[cfg(desktop)]
        {
            use runtime::desktop::display::vulkan::VulkanRendering;

            launch_gui::<VulkanRendering>(
                rom_manager,
                InitialGuiState::MainMenu,
                global_config.clone(),
            );
        }

        #[cfg(nintendo_3ds)]
        {
            // FIXME: Implement this with the gpu rendering plugins once that is ready
            launch_gui::<SoftwareRendering>(
                rom_manager,
                InitialGuiState::MainMenu,
                global_config.clone(),
            );
        }
    } else {
        launch_gui::<SoftwareRendering>(
            rom_manager,
            InitialGuiState::MainMenu,
            global_config.clone(),
        );
    }

    global_config.read().unwrap().save()?;

    Ok(())
}
