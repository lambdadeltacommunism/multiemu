use super::Component;
use crate::runtime::RenderingBackend;

pub trait DisplayComponent<R: RenderingBackend>: Component {
    fn initialize_display(&mut self, initialization_data: R::ComponentInitializationData);
    fn display_data(&self) -> &R::ComponentDisplayBuffer;
}
