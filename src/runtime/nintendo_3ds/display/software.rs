use super::Nintendo3dsRenderBackendState;
use crate::runtime::{RenderingBackend, RenderingBackendState};
use crate::{
    component::display::DisplayComponent, runtime::software_egui_render::SoftwareEguiRenderer,
};
use ctru::{
    prelude::Gfx,
    services::{
        gfx::{Flush, Screen, Swap},
        gspgpu::FramebufferFormat,
    },
};
use egui::{Context, FullOutput};
use nalgebra::{DMatrix, DMatrixViewMut, Matrix, Point2, Reflection2, Rotation2, Vector2};
use palette::{rgb::PackedBgra, Srgb, Srgba};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub struct SoftwareState {
    graphics_service: Rc<Gfx>,
    software_egui_renderer: SoftwareEguiRenderer,
}

impl RenderingBackendState for SoftwareState {
    type RenderingBackend = SoftwareRendering;

    fn redraw_egui(&mut self, context: &Context, full_output: FullOutput) {
        let mut top_screen = self.graphics_service.top_screen.borrow_mut();
        let screen_framebuffer = top_screen.raw_framebuffer();

        let screen_dimensions = Vector2::new(screen_framebuffer.height, screen_framebuffer.width);

        let mut screen_buffer = DMatrix::from_element(
            screen_dimensions.x,
            screen_dimensions.y,
            Srgba::new(0, 0, 0, 0xff),
        );

        self.software_egui_renderer.render(
            context,
            screen_buffer.view_range_mut(.., ..),
            full_output,
        );

        screen_buffer = screen_buffer.transpose();
        for i in 0..screen_dimensions.y / 2 {
            screen_buffer.swap_rows(i, screen_dimensions.y - i - 1);
        }

        let buffer_size = screen_dimensions.x * screen_dimensions.y * size_of::<PackedBgra>();
        // SAFETY: We set the buffer format ourselves so this should hold
        let surface_buffer_view: &mut [PackedBgra] = unsafe {
            std::slice::from_raw_parts_mut(screen_framebuffer.ptr as *mut PackedBgra, buffer_size)
        };

        for (i, pixel) in screen_buffer.into_iter().enumerate() {
            surface_buffer_view[i] = PackedBgra::from(*pixel);
        }

        top_screen.flush_buffers();
    }

    fn surface_resized(&mut self) {
        // Impossible on the 3ds
    }

    fn redraw(
        &mut self,
        display_components: &[Arc<Mutex<dyn DisplayComponent<Self::RenderingBackend>>>],
    ) {
        todo!()
    }

    fn initialize_components(
        &mut self,
        components: &[Arc<Mutex<dyn DisplayComponent<Self::RenderingBackend>>>],
    ) {
        todo!()
    }
}

impl Nintendo3dsRenderBackendState for SoftwareState {
    fn new() -> (Self, Rc<Gfx>) {
        let gfx = Rc::new(
            Gfx::with_formats_shared(FramebufferFormat::Rgba8, FramebufferFormat::Rgba8).unwrap(),
        );

        gfx.top_screen.borrow_mut().set_double_buffering(false);
        gfx.top_screen.borrow_mut().swap_buffers();

        (
            Self {
                graphics_service: gfx.clone(),
                software_egui_renderer: SoftwareEguiRenderer::default(),
            },
            gfx,
        )
    }
}

pub struct SoftwareRendering;

impl RenderingBackend for SoftwareRendering {
    type ComponentInitializationData = ();
    type ComponentDisplayBuffer = DMatrix<Srgba<u8>>;
    type RuntimeState = SoftwareState;
}
