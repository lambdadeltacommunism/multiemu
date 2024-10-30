use crate::{
    env::{CONFIG_LOCATION, STORAGE_DIRECTORY},
    input::keyboard::KeyboardInput,
};
use crate::{
    input::{Hotkey, Input},
    rom::{GameSystem, OtherSystem},
};
use indexmap::IndexMap;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_with::serde_as;
use std::{
    fs::{create_dir_all, File},
    ops::Deref,
    path::PathBuf,
};

#[serde_as]
#[serde_inline_default]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub controller_configs: IndexMap<GameSystem, IndexMap<Input, Input>>,
    #[serde(default)]
    pub hotkeys: IndexMap<Input, Hotkey>,
    #[serde_inline_default(true)]
    pub hardware_acceleration: bool,
    #[serde_inline_default(true)]
    pub vsync: bool,
    pub file_browser_home: PathBuf,
}

impl GlobalConfig {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        create_dir_all(STORAGE_DIRECTORY.deref())?;
        let config_file = File::create(CONFIG_LOCATION.deref())?;
        ron::ser::to_writer_pretty(config_file, self, PrettyConfig::default())?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config_file = File::open(CONFIG_LOCATION.deref())?;
        *self = ron::de::from_reader(config_file)?;

        Ok(())
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            controller_configs: [(
                GameSystem::Other(OtherSystem::Chip8),
                [
                    (
                        Input::Keyboard(KeyboardInput::Digit1),
                        Input::Keyboard(KeyboardInput::Numpad1),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::Digit2),
                        Input::Keyboard(KeyboardInput::Numpad2),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::Digit3),
                        Input::Keyboard(KeyboardInput::Numpad3),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::Digit4),
                        Input::Keyboard(KeyboardInput::KeyC),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyQ),
                        Input::Keyboard(KeyboardInput::Numpad4),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyW),
                        Input::Keyboard(KeyboardInput::Numpad5),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyE),
                        Input::Keyboard(KeyboardInput::Numpad6),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyR),
                        Input::Keyboard(KeyboardInput::KeyD),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyA),
                        Input::Keyboard(KeyboardInput::Numpad7),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyS),
                        Input::Keyboard(KeyboardInput::Numpad8),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyD),
                        Input::Keyboard(KeyboardInput::Numpad9),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyF),
                        Input::Keyboard(KeyboardInput::KeyE),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyZ),
                        Input::Keyboard(KeyboardInput::KeyA),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyX),
                        Input::Keyboard(KeyboardInput::Numpad0),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyC),
                        Input::Keyboard(KeyboardInput::KeyB),
                    ),
                    (
                        Input::Keyboard(KeyboardInput::KeyV),
                        Input::Keyboard(KeyboardInput::KeyF),
                    ),
                ]
                .into(),
            )]
            .into(),
            hotkeys: [(Input::Keyboard(KeyboardInput::F1), Hotkey::OpenMenu)].into(),
            hardware_acceleration: true,
            vsync: true,
            file_browser_home: STORAGE_DIRECTORY.clone(),
        }
    }
}
