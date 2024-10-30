use std::{path::PathBuf, sync::LazyLock};

#[cfg(desktop)]
pub static STORAGE_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().unwrap().join("multiemu"));
#[cfg(nintendo_3ds)]
pub static STORAGE_DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("sdmc:/multiemu"));

pub static CONFIG_LOCATION: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("config.ron"));
pub static LOG_LOCATION: LazyLock<PathBuf> = LazyLock::new(|| STORAGE_DIRECTORY.join("log.txt"));
pub static ROM_DATABASE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("database"));
pub static SAVE_RAM_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("saves"));
pub static SNAPSHOT_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("snapshot"));
pub static IMPORTED_ROM_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("roms"));
