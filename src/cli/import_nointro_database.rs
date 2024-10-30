use crate::{
    env::ROM_DATABASE_PATH,
    rom::{GameSystem, RomId, RomInfo, RomManager},
};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DefaultOnError;
use serde_with::DisplayFromStr;
use std::{fs::read_to_string, ops::Deref, path::PathBuf};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Datafile {
    header: Header,
    #[serde(alias = "game")]
    machine: Vec<Machine>,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct Header {
    #[serde_as(as = "DefaultOnError<DisplayFromStr>")]
    name: GameSystem,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Machine {
    #[serde(rename = "@name")]
    name: String,
    description: String,
    rom: Rom,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct Rom {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "@sha1")]
    hash: RomId,
    status: Option<String>,
    #[serde(rename = "@url")]
    url: Option<String>,
    #[serde(rename = "@region")]
    region: Option<String>,
}

pub fn run(files: Vec<PathBuf>) {
    let mut rom_manager = RomManager::default();
    let _ = rom_manager.load_rom_info(ROM_DATABASE_PATH.deref());

    for file in &files {
        let content = read_to_string(file).unwrap();

        // Parse XML based data file
        let data_file: Datafile = match quick_xml::de::from_str(&content) {
            Ok(file) => file,
            Err(err) => {
                tracing::error!(
                    "Failed to parse XML nointro database {}: {}",
                    file.display(),
                    err
                );
                continue;
            }
        };

        tracing::info!(
            "Found {} entries in nointro database {} for the system {}",
            data_file.machine.len(),
            file.display(),
            data_file.header.name
        );

        for game in data_file.machine.into_iter() {
            rom_manager.rom_information.insert(
                game.rom.hash,
                RomInfo {
                    name: Some(game.name),
                    hash: game.rom.hash,
                    system: data_file.header.name,
                    region: None,
                },
            );
        }
    }

    rom_manager
        .store_rom_info(ROM_DATABASE_PATH.deref())
        .unwrap();
}
