use super::Component;
use rmpv::Value;

pub trait SnapshotableComponent: Component {
    // Save the state of the component. Always run during a pause
    fn save_snapshot(&mut self) -> Value;

    // Load the state of the component. Always run durng a pause
    fn load_snapshot(&mut self, state: Value);
}
