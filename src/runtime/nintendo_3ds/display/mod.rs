use crate::runtime::RenderingBackendState;
use ctru::prelude::Gfx;
use std::{cell::RefCell, rc::Rc};

pub mod gpu;
pub mod software;

pub trait Nintendo3dsRenderBackendState: RenderingBackendState {
    fn new() -> (Self, Rc<Gfx>);
}
