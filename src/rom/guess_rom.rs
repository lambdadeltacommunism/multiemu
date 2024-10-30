use super::{AtariSystem, GameSystem, NintendoSystem, OtherSystem, RomId, RomManager, SegaSystem};
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::LazyLock,
};

struct MagicTableEntry {
    bytes: &'static [u8],
    offset: usize,
}

static MAGIC_TABLE: LazyLock<HashMap<GameSystem, Vec<MagicTableEntry>>> = LazyLock::new(|| {
    let mut table: HashMap<GameSystem, Vec<MagicTableEntry>> = HashMap::new();

    table
        .entry(GameSystem::Nintendo(NintendoSystem::GameBoy))
        .or_default()
        .extend([MagicTableEntry {
            bytes: &[0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b],
            offset: 0x134,
        }]);

    table
        .entry(GameSystem::Nintendo(
            NintendoSystem::NintendoEntertainmentSystem,
        ))
        .or_default()
        .extend([MagicTableEntry {
            bytes: &[b'N', b'E', b'S', 0x1a],
            offset: 0x00,
        }]);

    table
        .entry(GameSystem::Sega(SegaSystem::Genesis))
        .or_default()
        .extend([
            MagicTableEntry {
                bytes: b"SEGA GENESIS",
                offset: 0x100,
            },
            MagicTableEntry {
                bytes: b"SEGA MEGA DRIVE",
                offset: 0x100,
            },
        ]);

    table
        .entry(GameSystem::Sega(SegaSystem::MasterSystem))
        .or_default()
        .extend([
            MagicTableEntry {
                bytes: b"TMR SEGA",
                offset: 0x1ff0,
            },
            MagicTableEntry {
                bytes: b"TMR SEGA",
                offset: 0x3ff0,
            },
            MagicTableEntry {
                bytes: b"TMR SEGA",
                offset: 0x7ff0,
            },
        ]);

    table
});

pub fn guess_rom(rom: impl AsRef<Path>, rom_manager: &RomManager) -> Option<(GameSystem, RomId)> {
    let rom = rom.as_ref();
    let mut file = File::open(rom).ok()?;

    let mut hasher = Sha1::new();
    std::io::copy(&mut file, &mut hasher).unwrap();
    let hash = RomId::new(hasher.finalize().into());

    if let Some(system) = rom_manager.rom_information.get(&hash).map(|rom| rom.system) {
        tracing::info!(
            "Guessed system of ROM at {} from its hash and our database",
            rom.display()
        );

        return Some((system, hash));
    }

    // This goes first since a lot of roms have misleading or nonexistent magic bytes
    if let Some(value) = guess_by_extension(rom) {
        return Some((value, hash));
    }

    let mut read_buffer = Vec::new();

    for (system, entries) in MAGIC_TABLE.iter() {
        for entry in entries {
            read_buffer.resize(entry.bytes.len(), 0);

            if file.seek(SeekFrom::Start(entry.offset as u64)).is_err() {
                continue;
            }

            if file.read_exact(&mut read_buffer).is_err() {
                continue;
            }

            if read_buffer == entry.bytes {
                tracing::info!("Guessed system of ROM at {} from its magic", rom.display());

                return Some((*system, hash));
            }
        }
    }

    None
}

fn guess_by_extension(rom: &Path) -> Option<GameSystem> {
    if let Some(file_extension) = rom
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
    {
        if let Some(system) = match file_extension.as_str() {
            "gb" => Some(GameSystem::Nintendo(NintendoSystem::GameBoy)),
            "gbc" => Some(GameSystem::Nintendo(NintendoSystem::GameBoyColor)),
            "gba" => Some(GameSystem::Nintendo(NintendoSystem::GameBoyAdvance)),
            "nes" => Some(GameSystem::Nintendo(
                NintendoSystem::NintendoEntertainmentSystem,
            )),
            "sfc" | "smc" => Some(GameSystem::Nintendo(
                NintendoSystem::SuperNintendoEntertainmentSystem,
            )),
            "n64" | "z64" => Some(GameSystem::Nintendo(NintendoSystem::Nintendo64)),
            "md" => Some(GameSystem::Sega(SegaSystem::MasterSystem)),
            "gg" => Some(GameSystem::Sega(SegaSystem::GameGear)),
            "ch8" | "c8" => Some(GameSystem::Other(OtherSystem::Chip8)),
            "a26" => Some(GameSystem::Atari(AtariSystem::Atari2600)),
            _ => None,
        } {
            tracing::info!(
                "Guessed system of ROM at {} from file extension {}",
                rom.display(),
                file_extension
            );
            return Some(system);
        }
    }

    None
}
