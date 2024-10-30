use crate::{env::ROM_DATABASE_PATH, rom::RomManager};
use std::{ops::Deref, path::PathBuf};

pub fn run(directory: Vec<PathBuf>) {
    let mut rom_manager = RomManager::default();
    let _ = rom_manager.load_rom_info(ROM_DATABASE_PATH.deref());

    for path in &directory {
        rom_manager.load_rom_info(path).unwrap();
    }

    rom_manager
        .store_rom_info(ROM_DATABASE_PATH.deref())
        .unwrap();
}
