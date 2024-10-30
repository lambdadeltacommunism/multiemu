use super::Component;
use crate::input::{EmulatedGamepad, Input};
use std::sync::Arc;

pub trait InputComponent: Component {
    fn registered_inputs(&self) -> &'static [Input];
    fn assign_controller(&mut self, controller: Arc<EmulatedGamepad>);
}
