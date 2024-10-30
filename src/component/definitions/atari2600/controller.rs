use crate::{
    component::{input::InputComponent, Component},
    input::{gamepad::GamepadInput, EmulatedGamepad, Input},
};
use std::sync::Arc;

pub struct Atari2600Controller {
    assigned_controller: Option<Arc<EmulatedGamepad>>,
}

impl Component for Atari2600Controller {}

impl InputComponent for Atari2600Controller {
    fn registered_inputs(&self) -> &'static [Input] {
        &[
            // Joystick
            Input::Gamepad(GamepadInput::LeftStickUp),
            Input::Gamepad(GamepadInput::LeftStickDown),
            Input::Gamepad(GamepadInput::LeftStickLeft),
            Input::Gamepad(GamepadInput::LeftStickRight),
            // A button
            Input::Gamepad(GamepadInput::FPadDown),
        ]
    }

    fn assign_controller(&mut self, controller: Arc<EmulatedGamepad>) {
        self.assigned_controller = Some(controller);
    }
}
