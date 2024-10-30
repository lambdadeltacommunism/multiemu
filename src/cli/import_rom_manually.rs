use crate::{
    env::ROM_DATABASE_PATH,
    rom::{GameSystem, RomId, RomInfo, RomManager},
};
use sha1::{Digest, Sha1};
use std::{fs, ops::Deref, path::PathBuf};

pub fn run(file: PathBuf, system: GameSystem, name: String) {
    let mut rom_manager = RomManager::default();
    let _ = rom_manager.load_rom_info(ROM_DATABASE_PATH.deref());

    let mut hasher = Sha1::default();
    let mut rom = fs::File::open(&file).unwrap();
    std::io::copy(&mut rom, &mut hasher).unwrap();
    let hash = RomId::new(hasher.finalize().into());

    tracing::info!("Imported ROM {} with hash {}", name, hash);

    rom_manager.rom_information.insert(
        hash,
        RomInfo {
            name: Some(name),
            system,
            hash,
            region: None,
        },
    );

    rom_manager
        .store_rom_info(ROM_DATABASE_PATH.deref())
        .unwrap();
}
