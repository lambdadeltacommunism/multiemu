use gamepad::GamepadInput;
use keyboard::KeyboardInput;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod gamepad;
pub mod keyboard;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    Gamepad(GamepadInput),
    // In game uses key codes
    Keyboard(KeyboardInput),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputState {
    /// 0 or 1
    Digital(bool),
    /// Clamped from 0.0 to 1.0
    Analog(f32),
}

impl Default for InputState {
    fn default() -> Self {
        Self::Digital(false)
    }
}

impl InputState {
    pub fn as_digital(&self) -> bool {
        match self {
            InputState::Digital(value) => *value,
            InputState::Analog(value) => *value >= 0.5,
        }
    }

    pub fn as_analog(&self) -> f32 {
        match self {
            InputState::Digital(value) => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            InputState::Analog(value) => *value,
        }
    }
}

#[derive(Debug)]
pub struct EmulatedGamepad(Mutex<HashMap<Input, InputState>>);

impl EmulatedGamepad {
    pub fn new(inputs: &[Input]) -> Arc<Self> {
        let mut map = HashMap::new();
        for input in inputs {
            map.insert(*input, InputState::Digital(false));
        }
        Arc::new(Self(Mutex::new(map)))
    }

    pub fn set_input_state(&self, input: Input, input_state: InputState) {
        if let Some(value) = self.0.lock().unwrap().get_mut(&input) {
            *value = input_state;
        }
    }

    pub fn get_input_state(&self, input: Input) -> Option<InputState> {
        self.0.lock().unwrap().get(&input).copied()
    }

    pub fn iter_pressed(&self) -> impl Iterator<Item = Input> + '_ {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(input, state)| {
                if state.as_digital() {
                    Some(*input)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn iter_released(&self) -> impl Iterator<Item = Input> + '_ {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(input, state)| {
                if !state.as_digital() {
                    Some(*input)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Hotkey {
    OpenMenu,
}
