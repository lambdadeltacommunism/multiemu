use crate::{
    config::GlobalConfig,
    rom::{GameSystem, RomId},
};
use clap::{Parser, Subcommand, ValueEnum};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub mod import_known_roms;
pub mod import_native_database;
pub mod import_nointro_database;
pub mod import_rom_manually;
pub mod run_external_rom;
pub mod run_rom;

#[derive(ValueEnum, Clone, Debug)]
pub enum DatabaseType {
    Native,
    Nointro,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<CliAction>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CliAction {
    ImportDatabase {
        database_type: DatabaseType,
        #[arg(required=true, num_args=1..)]
        path: Vec<PathBuf>,
    },
    ImportRomManually {
        system: GameSystem,
        name: String,
        path: PathBuf,
    },
    ImportKnownRoms {
        #[clap(short, long)]
        symlink: bool,
        #[arg(required=true, num_args=1..)]
        path: Vec<PathBuf>,
    },
    VerifyRoms {
        #[clap(short, long)]
        unknown_discard: bool,
        #[clap(short, long)]
        incorrect_discard: bool,
    },
    Run {
        #[clap(short, long)]
        force_system: Option<GameSystem>,
        #[arg(required=true, num_args=1..)]
        rom: Vec<RomId>,
    },
    RunExternal {
        #[clap(short, long)]
        force_system: Option<GameSystem>,
        #[arg(required=true, num_args=1..)]
        rom: Vec<PathBuf>,
    },
}

pub fn handle_cli(cli_action: CliAction, global_config: Arc<RwLock<GlobalConfig>>) {
    match cli_action {
        CliAction::ImportDatabase {
            database_type: DatabaseType::Native,
            path,
        } => {
            import_native_database::run(path);
        }
        CliAction::ImportDatabase {
            database_type: DatabaseType::Nointro,
            path,
        } => {
            import_nointro_database::run(path);
        }
        CliAction::Run { rom, force_system } => {
            if force_system.is_some() {
                tracing::warn!(
                    "Forcing a system is not recommended as it can cause mysterious problems"
                );
            }

            run_rom::run(rom, global_config);
        }
        CliAction::RunExternal { rom, force_system } => {
            if force_system.is_some() {
                tracing::warn!(
                    "Forcing a system is not recommended as it can cause mysterious problems"
                );
            }

            run_external_rom::run(rom, force_system, global_config);
        }

        CliAction::ImportRomManually { path, system, name } => {
            import_rom_manually::run(path, system, name);
        }
        CliAction::ImportKnownRoms { path, symlink } => {
            import_known_roms::run(path, symlink);
        }
        CliAction::VerifyRoms {
            unknown_discard,
            incorrect_discard,
        } => todo!(),
    }
}
