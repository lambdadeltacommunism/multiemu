use crate::{
    config::GlobalConfig,
    env::{IMPORTED_ROM_DIRECTORY, ROM_DATABASE_PATH},
    rom::{RomId, RomManager},
    runtime::{
        desktop::display::vulkan::VulkanRendering, launch_gui, InitialGuiState, SoftwareRendering,
    },
};
use std::{
    fs::create_dir_all,
    ops::Deref,
    sync::{Arc, RwLock},
};

pub fn run(user_specified_roms: Vec<RomId>, global_config: Arc<RwLock<GlobalConfig>>) {
    let mut rom_manager = RomManager::default();

    create_dir_all(IMPORTED_ROM_DIRECTORY.deref()).unwrap();

    rom_manager
        .load_rom_info(ROM_DATABASE_PATH.deref())
        .unwrap();
    rom_manager
        .load_rom_paths(IMPORTED_ROM_DIRECTORY.deref())
        .unwrap();

    for rom_id in &user_specified_roms {
        if !rom_manager.rom_paths.contains_key(rom_id) {
            tracing::error!("ROM {} not found", rom_id);
            return;
        }
    }

    let rom_manager = Arc::new(rom_manager);
    let game_system = rom_manager.rom_information[&user_specified_roms[0]].system;

    if global_config.read().unwrap().hardware_acceleration {
        launch_gui::<VulkanRendering>(rom_manager, InitialGuiState::MainMenu, global_config);
    } else {
        launch_gui::<SoftwareRendering>(rom_manager, InitialGuiState::MainMenu, global_config);
    }
}
