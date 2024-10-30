use crate::{
    config::GlobalConfig,
    input::{gamepad::GamepadInput, EmulatedGamepad, Input, InputState},
    rom::GameSystem,
};
use arrayvec::ArrayVec;
use gilrs::{Axis, Button, EventType, Gilrs};
use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
};

pub struct GilrsGamepadManager {
    context: Gilrs,
    gamepads: Vec<Arc<EmulatedGamepad>>,
    system: GameSystem,
    global_config: Arc<RwLock<GlobalConfig>>,
}

impl GilrsGamepadManager {
    pub fn new(
        gamepads: Vec<Arc<EmulatedGamepad>>,
        system: GameSystem,
        global_config: Arc<RwLock<GlobalConfig>>,
    ) -> Self {
        Self {
            context: Gilrs::new().unwrap(),
            gamepads,
            system,
            global_config,
        }
    }

    pub fn insert_input(&mut self, input: Input, input_state: InputState) {
        if let Some(translated_input) = self
            .global_config
            .read()
            .unwrap()
            .controller_configs
            .get(&self.system)
            .and_then(|config| config.get(&input))
            .copied()
        {
            self.gamepads[0].set_input_state(translated_input, input_state);
        }
    }

    pub fn refresh_gamepad_inputs(&mut self) {
        while let Some(event) = self.context.next_event() {
            match event.event {
                EventType::AxisChanged(axis, value, _) => {
                    for (axis, value) in gilrs_axis_translator(axis, value) {
                        self.insert_input(axis, value);
                    }
                }
                EventType::ButtonChanged(button, value, _) => {
                    if let Some(button) = gilrs_button_translator(button) {
                        self.insert_input(button, InputState::Analog(value));
                    }
                }
                _ => {}
            }
        }
    }
}

#[inline]
fn gilrs_button_translator(button: Button) -> Option<Input> {
    // TODO: think about these mappings a little harder
    Some(match button {
        Button::South => Input::Gamepad(GamepadInput::FPadDown),
        Button::East => Input::Gamepad(GamepadInput::FPadRight),
        Button::North => Input::Gamepad(GamepadInput::FPadUp),
        Button::West => Input::Gamepad(GamepadInput::FPadLeft),
        Button::Z => Input::Gamepad(GamepadInput::ZTrigger),
        Button::LeftTrigger => Input::Gamepad(GamepadInput::LeftTrigger),
        Button::LeftTrigger2 => Input::Gamepad(GamepadInput::LeftSecondaryTrigger),
        Button::RightTrigger => Input::Gamepad(GamepadInput::RightTrigger),
        Button::RightTrigger2 => Input::Gamepad(GamepadInput::RightSecondaryTrigger),
        Button::Select => Input::Gamepad(GamepadInput::Select),
        Button::Start => Input::Gamepad(GamepadInput::Start),
        Button::Mode => Input::Gamepad(GamepadInput::Mode),
        Button::LeftThumb => Input::Gamepad(GamepadInput::LeftThumb),
        Button::RightThumb => Input::Gamepad(GamepadInput::RightThumb),
        Button::DPadUp => Input::Gamepad(GamepadInput::DPadUp),
        Button::DPadDown => Input::Gamepad(GamepadInput::DPadDown),
        Button::DPadLeft => Input::Gamepad(GamepadInput::DPadLeft),
        Button::DPadRight => Input::Gamepad(GamepadInput::DPadRight),
        Button::C => todo!(),
        Button::Unknown => {
            tracing::warn!("Unknown button pressed");
            return None;
        }
    })
}

#[inline]
fn gilrs_axis_translator(axis: Axis, value: f32) -> ArrayVec<(Input, InputState), 2> {
    let value = value.clamp(-1.0, 1.0);

    match axis {
        Axis::LeftStickX => match value.total_cmp(&0.0) {
            Ordering::Less => [
                (
                    Input::Gamepad(GamepadInput::LeftStickLeft),
                    InputState::Analog(value.abs()),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickRight),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Equal => [
                (
                    Input::Gamepad(GamepadInput::LeftStickLeft),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickRight),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Greater => [
                (
                    Input::Gamepad(GamepadInput::LeftStickLeft),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickRight),
                    InputState::Analog(value.abs()),
                ),
            ]
            .into(),
        },
        Axis::LeftStickY => match value.total_cmp(&0.0) {
            Ordering::Less => [
                (
                    Input::Gamepad(GamepadInput::LeftStickUp),
                    InputState::Analog(value.abs()),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickDown),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Equal => [
                (
                    Input::Gamepad(GamepadInput::LeftStickUp),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickDown),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Greater => [
                (
                    Input::Gamepad(GamepadInput::LeftStickUp),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::LeftStickDown),
                    InputState::Analog(value.abs()),
                ),
            ]
            .into(),
        },
        Axis::LeftZ => todo!(),
        Axis::RightStickX => match value.total_cmp(&0.0) {
            Ordering::Less => [
                (
                    Input::Gamepad(GamepadInput::RightStickLeft),
                    InputState::Analog(value.abs()),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickRight),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Equal => [
                (
                    Input::Gamepad(GamepadInput::RightStickLeft),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickRight),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Greater => [
                (
                    Input::Gamepad(GamepadInput::RightStickLeft),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickRight),
                    InputState::Analog(value.abs()),
                ),
            ]
            .into(),
        },
        Axis::RightStickY => match value.total_cmp(&0.0) {
            Ordering::Less => [
                (
                    Input::Gamepad(GamepadInput::RightStickUp),
                    InputState::Analog(value.abs()),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickDown),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Equal => [
                (
                    Input::Gamepad(GamepadInput::RightStickUp),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickDown),
                    InputState::Analog(0.0),
                ),
            ]
            .into(),
            Ordering::Greater => [
                (
                    Input::Gamepad(GamepadInput::RightStickUp),
                    InputState::Analog(0.0),
                ),
                (
                    Input::Gamepad(GamepadInput::RightStickDown),
                    InputState::Analog(value.abs()),
                ),
            ]
            .into(),
        },
        _ => ArrayVec::new(),
    }
}
