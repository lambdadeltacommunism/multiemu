use crate::{
    config::GlobalConfig,
    env::{IMPORTED_ROM_DIRECTORY, ROM_DATABASE_PATH},
    rom::{guess_rom::guess_rom, GameSystem, RomId, RomInfo, RomManager},
    runtime::{
        desktop::display::vulkan::VulkanRendering, launch_gui, InitialGuiState, SoftwareRendering,
    },
};
use sha1::{Digest, Sha1};
use std::{
    fs::{create_dir_all, File},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub fn run(
    roms: Vec<PathBuf>,
    force_system: Option<GameSystem>,
    global_config: Arc<RwLock<GlobalConfig>>,
) {
    for rom in &roms {
        if !rom.is_file() {
            panic!("Rom at {} is not a file", rom.display());
        }
    }

    let mut rom_manager = RomManager::default();

    create_dir_all(IMPORTED_ROM_DIRECTORY.deref()).unwrap();

    rom_manager
        .load_rom_info(ROM_DATABASE_PATH.deref())
        .unwrap();
    rom_manager
        .load_rom_paths(IMPORTED_ROM_DIRECTORY.deref())
        .unwrap();

    let mut user_specified_roms = Vec::new();

    let mut game_system = None;

    if let Some(forced_game_system) = force_system {
        for rom_path in &roms {
            let mut file = File::open(rom_path).unwrap();
            let mut hasher = Sha1::new();
            std::io::copy(&mut file, &mut hasher).unwrap();
            let hash = RomId::new(hasher.finalize().into());
            rom_manager.rom_paths.insert(hash, rom_path.clone());
            rom_manager.rom_information.insert(
                hash,
                RomInfo {
                    name: None,
                    hash,
                    system: forced_game_system,
                    region: None,
                },
            );
            user_specified_roms.push(hash);
        }

        game_system = Some(forced_game_system);
    } else {
        for rom_path in &roms {
            let Some((guessed_game_system, rom_id)) = guess_rom(rom_path, &rom_manager) else {
                panic!("Failed to guess system for {}", rom_path.display());
            };

            if let Some(game_system) = game_system {
                if guessed_game_system != game_system {
                    panic!(
                        "ROM has confusing system specification: expected {} but got {}",
                        game_system, guessed_game_system
                    );
                }
            } else {
                game_system = Some(guessed_game_system);
            }

            rom_manager.rom_paths.insert(rom_id, rom_path.clone());
            rom_manager.rom_information.insert(
                rom_id,
                RomInfo {
                    name: None,
                    hash: rom_id,
                    system: guessed_game_system,
                    region: None,
                },
            );
            user_specified_roms.push(rom_id);
        }
    }

    let rom_manager = Arc::new(rom_manager);
    let game_system = game_system.expect("Failed to guess game system");

    if global_config.read().unwrap().hardware_acceleration {
        launch_gui::<VulkanRendering>(
            rom_manager,
            InitialGuiState::OpenGame {
                user_specified_roms,
                game_system,
            },
            global_config,
        );
    } else {
        launch_gui::<SoftwareRendering>(
            rom_manager,
            InitialGuiState::OpenGame {
                user_specified_roms,
                game_system,
            },
            global_config,
        );
    }
}
