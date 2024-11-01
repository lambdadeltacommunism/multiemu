use data_encoding::HEXLOWER_PERMISSIVE;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
    str::FromStr,
};
use std::{fmt::Display, path::Path};
use strum::{EnumIter, IntoEnumIterator};

pub mod guess_rom;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum NintendoSystem {
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    GameCube,
    SuperNintendoEntertainmentSystem,
    NintendoEntertainmentSystem,
    Nintendo64,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum SegaSystem {
    MasterSystem,
    GameGear,
    Genesis,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum SonySystem {
    Playstation,
    Playstation2,
    Playstation3,
    PlaystationPortable,
    PlaystationVita,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum OtherSystem {
    Chip8,
    SuperChip8,
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter,
)]
pub enum AtariSystem {
    Atari2600,
}

#[derive(
    Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum GameSystem {
    Nintendo(NintendoSystem),
    Sega(SegaSystem),
    Sony(SonySystem),
    Atari(AtariSystem),
    Nec,
    Microsoft,
    Commodore,
    Snk,
    Bandai,
    Other(OtherSystem),
    #[default]
    Unknown,
}

impl GameSystem {
    pub fn iter() -> impl Iterator<Item = GameSystem> {
        NintendoSystem::iter()
            .map(GameSystem::Nintendo)
            .chain(SegaSystem::iter().map(GameSystem::Sega))
            .chain(SonySystem::iter().map(GameSystem::Sony))
            .chain(AtariSystem::iter().map(GameSystem::Atari))
            .chain(OtherSystem::iter().map(GameSystem::Other))
    }
}

impl FromStr for GameSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "nintendo - game boy" | "nintendo - gameboy" | "nintendo - gb" => {
                Ok(GameSystem::Nintendo(NintendoSystem::GameBoy))
            }
            "nintendo - game boy color" | "nintendo - gameboy color" | "nintendo - gbc" => {
                Ok(GameSystem::Nintendo(NintendoSystem::GameBoyColor))
            }
            "nintendo - game boy advance" | "nintendo - gameboy advance" | "nintendo - gba" => {
                Ok(GameSystem::Nintendo(NintendoSystem::GameBoyAdvance))
            }
            "nintendo - game cube" | "nintendo - gamecube" => {
                Ok(GameSystem::Nintendo(NintendoSystem::GameCube))
            }
            "nintendo - super nintendo entertainment system" | "nintendo - snes" => Ok(
                GameSystem::Nintendo(NintendoSystem::SuperNintendoEntertainmentSystem),
            ),
            "nintendo - nintendo entertainment system" | "nintendo - nes" => Ok(
                GameSystem::Nintendo(NintendoSystem::NintendoEntertainmentSystem),
            ),
            "nintendo - nintendo 64" | "nintendo - n64" => {
                Ok(GameSystem::Nintendo(NintendoSystem::Nintendo64))
            }
            "sega - master system" | "sega - ms" => Ok(GameSystem::Sega(SegaSystem::MasterSystem)),
            "sega - game gear" | "sega - gg" => Ok(GameSystem::Sega(SegaSystem::GameGear)),
            "sega - genesis" | "sega - ge" | "sega - megadrive" | "sega - md" => {
                Ok(GameSystem::Sega(SegaSystem::Genesis))
            }
            "sony - playstation" | "sony - ps" | "sony - ps1" | "sony - psx" => {
                Ok(GameSystem::Sony(SonySystem::Playstation))
            }
            "sony - playstation 2" | "sony - ps2" => Ok(GameSystem::Sony(SonySystem::Playstation2)),
            "sony - playstation 3" | "sony - ps3" => Ok(GameSystem::Sony(SonySystem::Playstation3)),
            "sony - playstation portable" | "sony - psp" => {
                Ok(GameSystem::Sony(SonySystem::PlaystationPortable))
            }
            "sony - playstation vita" => Ok(GameSystem::Sony(SonySystem::PlaystationVita)),
            "other - chip8" => Ok(GameSystem::Other(OtherSystem::Chip8)),
            "other - super chip8" => Ok(GameSystem::Other(OtherSystem::SuperChip8)),
            "atari - atari 2600" | "atari - 2600" => Ok(GameSystem::Atari(AtariSystem::Atari2600)),
            _ => Err(format!("Unknown system: {}", s)),
        }
    }
}

impl Display for GameSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameSystem::Nintendo(NintendoSystem::GameBoy) => write!(f, "Nintendo - Game Boy"),
            GameSystem::Nintendo(NintendoSystem::GameBoyColor) => {
                write!(f, "Nintendo - Game Boy Color")
            }
            GameSystem::Nintendo(NintendoSystem::GameBoyAdvance) => {
                write!(f, "Nintendo - Game Boy Advance")
            }
            GameSystem::Nintendo(NintendoSystem::GameCube) => write!(f, "Nintendo - GameCube"),
            GameSystem::Nintendo(NintendoSystem::SuperNintendoEntertainmentSystem) => {
                write!(f, "Nintendo - Super Nintendo Entertainment System")
            }
            GameSystem::Nintendo(NintendoSystem::NintendoEntertainmentSystem) => {
                write!(f, "Nintendo - Nintendo Entertainment System")
            }
            GameSystem::Nintendo(NintendoSystem::Nintendo64) => write!(f, "Nintendo - Nintendo 64"),
            GameSystem::Sony(SonySystem::Playstation) => write!(f, "Sony - PlayStation"),
            GameSystem::Sony(SonySystem::Playstation2) => write!(f, "Sony - PlayStation 2"),
            GameSystem::Sony(SonySystem::Playstation3) => write!(f, "Sony - PlayStation 3"),
            GameSystem::Sony(SonySystem::PlaystationPortable) => {
                write!(f, "Sony - PlayStation Portable")
            }
            GameSystem::Sony(SonySystem::PlaystationVita) => write!(f, "Sony - PlayStation Vita"),
            GameSystem::Sega(SegaSystem::MasterSystem) => write!(f, "Sega - Master System"),
            GameSystem::Sega(SegaSystem::GameGear) => write!(f, "Sega - Game Gear"),
            GameSystem::Sega(SegaSystem::Genesis) => write!(f, "Sega - Genesis"),
            GameSystem::Other(OtherSystem::Chip8) => write!(f, "Other - Chip8"),
            GameSystem::Other(OtherSystem::SuperChip8) => write!(f, "Other - Super Chip8"),
            GameSystem::Atari(AtariSystem::Atari2600) => write!(f, "Atari - 2600"),
            GameSystem::Nec => todo!(),
            GameSystem::Microsoft => todo!(),
            GameSystem::Commodore => todo!(),
            GameSystem::Snk => todo!(),
            GameSystem::Bandai => todo!(),
            GameSystem::Unknown => write!(f, "Unknown"),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RomInfo {
    pub name: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub hash: RomId,
    pub system: GameSystem,
    pub region: Option<RomRegion>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RomRegion {
    World,
    Japan,
    Europe,
    NorthAmerica,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Sha-1 of rom
pub struct RomId([u8; 20]);

impl RomId {
    pub const fn new(data: [u8; 20]) -> Self {
        Self(data)
    }
}

impl AsRef<[u8]> for RomId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 20]> for RomId {
    fn from(value: [u8; 20]) -> Self {
        Self(value)
    }
}

impl Display for RomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", HEXLOWER_PERMISSIVE.encode(&self.0))
    }
}

impl FromStr for RomId {
    type Err = data_encoding::DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = HEXLOWER_PERMISSIVE.decode(s.as_bytes())?;
        Ok(Self(bytes.try_into().unwrap()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RomRequirement {
    /// Ok to boot machine without this ROM but runtime failure can occur without it
    Sometimes,
    /// Machine will boot emulating this ROM
    Optional,
    /// Machine can not boot without this ROM
    Required,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct RomManager {
    pub rom_information: HashMap<RomId, RomInfo>,
    pub rom_paths: HashMap<RomId, PathBuf>,
}

impl RomManager {
    pub fn load_rom_info(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();

        if !path.is_file() {
            return Err("Path is not a file".into());
        }

        let file = BufReader::new(File::open(path)?);
        let datasheet: Vec<RomInfo> = rmp_serde::from_read(file)?;
        self.rom_information
            .extend(datasheet.into_iter().map(|info| (info.hash, info)));

        Ok(())
    }

    pub fn store_rom_info(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let rom_info = self.rom_information.values().cloned().collect::<Vec<_>>();

        let mut file = BufWriter::new(File::create(path)?);
        rmp_serde::encode::write_named(&mut file, &rom_info)?;

        Ok(())
    }

    pub fn load_rom_paths(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();

        let roms = fs::read_dir(path)?;

        for rom in roms {
            let rom = rom?;
            let path = rom.path();

            if !path.is_file() {
                continue;
            }

            let path_name: RomId = path.file_name().unwrap().to_str().unwrap().parse()?;

            self.rom_paths.insert(path_name, path);
        }

        Ok(())
    }

    pub fn load_rom_paths_verified(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<HashMap<RomId, PathBuf>, Box<dyn Error>> {
        let path = path.as_ref();

        let roms = fs::read_dir(path)?;

        let mut incorrect_roms = HashMap::new();

        for rom in roms {
            let rom = rom?;
            let path = rom.path();

            if !path.is_file() {
                continue;
            }

            let expected_hash = path.file_name().unwrap().to_str().unwrap().parse()?;

            let mut file = File::open(&path)?;
            let mut hasher = Sha1::new();
            std::io::copy(&mut file, &mut hasher)?;
            let hash = RomId::new(hasher.finalize().into());

            if hash != expected_hash {
                incorrect_roms.insert(hash, path);
            } else {
                self.rom_paths.insert(hash, path);
            }
        }

        Ok(incorrect_roms)
    }

    /// Components should use this function to load roms for themselves
    pub fn open(&self, id: RomId, requirement: RomRequirement) -> Option<File> {
        if let Some(path) = self.rom_paths.get(&id) {
            return File::open(path).ok();
        }

        match requirement {
            RomRequirement::Sometimes => {
                tracing::warn!(
                    "Could not find ROM {} for machine, machine will continue in a degraded state",
                    id
                );

                None
            }
            RomRequirement::Optional => {
                tracing::info!(
                    "Could not find ROM {} for machine, but it's optional for runtime",
                    id
                );

                None
            }
            RomRequirement::Required => {
                tracing::error!("ROM {} is required for machine, but not found", id);

                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RomSpecification {
    Path(PathBuf),
    Hash(RomId),
}

impl From<PathBuf> for RomSpecification {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<RomId> for RomSpecification {
    fn from(hash: RomId) -> Self {
        Self::Hash(hash)
    }
}
