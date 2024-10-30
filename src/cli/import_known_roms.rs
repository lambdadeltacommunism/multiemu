use crate::{
    env::{IMPORTED_ROM_DIRECTORY, ROM_DATABASE_PATH},
    rom::{RomId, RomManager},
};
use sha1::{Digest, Sha1};
use std::{
    fs::{self, copy, create_dir_all, File},
    ops::Deref,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn run(paths: Vec<PathBuf>, symlink: bool) {
    let mut rom_manager = RomManager::default();
    rom_manager
        .load_rom_info(ROM_DATABASE_PATH.deref())
        .expect("Cannot load ROM database");

    create_dir_all(IMPORTED_ROM_DIRECTORY.deref()).unwrap();

    for path in paths {
        if path.is_dir() {
            let walkdir = WalkDir::new(path);

            for path in walkdir.into_iter().flatten() {
                process_file(&rom_manager, symlink, path.path());
            }
        } else {
            process_file(&rom_manager, symlink, path);
        }
    }
}

fn process_file(rom_manager: &RomManager, symlink: bool, path: impl AsRef<Path>) {
    let path = path.as_ref();

    if path.is_dir() {
        return;
    }

    let mut file = File::open(path).unwrap();
    let mut hasher = Sha1::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    let hash = RomId::new(hasher.finalize().into());

    if let Some(rom) = rom_manager.rom_information.get(&hash) {
        let hash_string = hash.to_string();

        tracing::info!(
            "Identified ROM at {} as \"{:?}\" for the system {} with hash {}",
            path.display(),
            rom.name,
            rom.system,
            hash_string
        );
        let internal_store_path = IMPORTED_ROM_DIRECTORY.join(hash_string);
        let _ = fs::remove_file(&internal_store_path);

        #[cfg(unix)]
        if symlink {
            std::os::unix::fs::symlink(path, internal_store_path).unwrap();
        } else {
            copy(path, internal_store_path).unwrap();
        }

        #[cfg(windows)]
        if symlink {
            std::os::windows::fs::symlink_file(path, internal_store_path).unwrap();
        } else {
            copy(path, internal_store_path).unwrap();
        }

        #[cfg(not(any(unix, windows)))]
        if symlink {
            panic!("Symlinking is not supported on this platform");
        } else {
            copy(&path, internal_store_path).unwrap();
        }
    }
}
