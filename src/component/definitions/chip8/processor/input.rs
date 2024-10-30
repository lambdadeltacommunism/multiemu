use crate::input::{keyboard::KeyboardInput, Input};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chip8Key(pub u8);

impl TryFrom<Input> for Chip8Key {
    type Error = ();

    fn try_from(value: Input) -> Result<Self, Self::Error> {
        match value {
            Input::Keyboard(KeyboardInput::Numpad0) => Ok(Chip8Key(0x0)),
            Input::Keyboard(KeyboardInput::Numpad1) => Ok(Chip8Key(0x1)),
            Input::Keyboard(KeyboardInput::Numpad2) => Ok(Chip8Key(0x2)),
            Input::Keyboard(KeyboardInput::Numpad3) => Ok(Chip8Key(0x3)),
            Input::Keyboard(KeyboardInput::Numpad4) => Ok(Chip8Key(0x4)),
            Input::Keyboard(KeyboardInput::Numpad5) => Ok(Chip8Key(0x5)),
            Input::Keyboard(KeyboardInput::Numpad6) => Ok(Chip8Key(0x6)),
            Input::Keyboard(KeyboardInput::Numpad7) => Ok(Chip8Key(0x7)),
            Input::Keyboard(KeyboardInput::Numpad8) => Ok(Chip8Key(0x8)),
            Input::Keyboard(KeyboardInput::Numpad9) => Ok(Chip8Key(0x9)),
            Input::Keyboard(KeyboardInput::KeyA) => Ok(Chip8Key(0xa)),
            Input::Keyboard(KeyboardInput::KeyB) => Ok(Chip8Key(0xb)),
            Input::Keyboard(KeyboardInput::KeyC) => Ok(Chip8Key(0xc)),
            Input::Keyboard(KeyboardInput::KeyD) => Ok(Chip8Key(0xd)),
            Input::Keyboard(KeyboardInput::KeyE) => Ok(Chip8Key(0xe)),
            Input::Keyboard(KeyboardInput::KeyF) => Ok(Chip8Key(0xf)),
            _ => Err(()),
        }
    }
}

impl TryFrom<Chip8Key> for Input {
    type Error = ();

    fn try_from(value: Chip8Key) -> Result<Self, Self::Error> {
        match value.0 {
            0x0 => Ok(Input::Keyboard(KeyboardInput::Numpad0)),
            0x1 => Ok(Input::Keyboard(KeyboardInput::Numpad1)),
            0x2 => Ok(Input::Keyboard(KeyboardInput::Numpad2)),
            0x3 => Ok(Input::Keyboard(KeyboardInput::Numpad3)),
            0x4 => Ok(Input::Keyboard(KeyboardInput::Numpad4)),
            0x5 => Ok(Input::Keyboard(KeyboardInput::Numpad5)),
            0x6 => Ok(Input::Keyboard(KeyboardInput::Numpad6)),
            0x7 => Ok(Input::Keyboard(KeyboardInput::Numpad7)),
            0x8 => Ok(Input::Keyboard(KeyboardInput::Numpad8)),
            0x9 => Ok(Input::Keyboard(KeyboardInput::Numpad9)),
            0xa => Ok(Input::Keyboard(KeyboardInput::KeyA)),
            0xb => Ok(Input::Keyboard(KeyboardInput::KeyB)),
            0xc => Ok(Input::Keyboard(KeyboardInput::KeyC)),
            0xd => Ok(Input::Keyboard(KeyboardInput::KeyD)),
            0xe => Ok(Input::Keyboard(KeyboardInput::KeyE)),
            0xf => Ok(Input::Keyboard(KeyboardInput::KeyF)),
            _ => Err(()),
        }
    }
}
