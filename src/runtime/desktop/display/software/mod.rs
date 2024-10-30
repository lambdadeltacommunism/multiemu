use super::WinitRenderBackendState;
use crate::{
    component::display::DisplayComponent,
    config::GlobalConfig,
    runtime::{
        software_egui_render::SoftwareEguiRenderer, RenderingBackend, RenderingBackendState,
    },
};
use egui::FullOutput;
use nalgebra::{DMatrix, DMatrixViewMut, Vector2};
use palette::Srgba;
use softbuffer::{Context, Surface};
use std::sync::{Arc, Mutex, RwLock};
use winit::window::Window;

pub struct SoftwareState {
    surface: Surface<Arc<Window>, Arc<Window>>,
    window: Arc<Window>,
    global_config: Arc<RwLock<GlobalConfig>>,
    egui_renderer: SoftwareEguiRenderer,
}

impl RenderingBackendState for SoftwareState {
    type RenderingBackend = SoftwareRendering;

    fn redraw_egui(&mut self, context: &egui::Context, full_output: FullOutput) {
        let window_dimensions = Vector2::new(
            self.window.inner_size().width,
            self.window.inner_size().height,
        )
        .cast();

        // Lock the buffer before we start drawing
        let mut surface_buffer = self.surface.buffer_mut().unwrap();

        // TODO: Probably breaks on little endian processors
        let surface_buffer_view: &mut [Srgba<u8>] =
            bytemuck::cast_slice_mut(surface_buffer.as_mut());
        let surface_buffer_view = DMatrixViewMut::from_slice(
            surface_buffer_view,
            window_dimensions.x,
            window_dimensions.y,
        );

        self.egui_renderer
            .render(context, surface_buffer_view, full_output);

        // Present
        surface_buffer.present().unwrap();
    }

    fn surface_resized(&mut self) {
        let [window_width, window_height]: [u32; 2] = self.window.inner_size().into();

        self.surface
            .resize(
                window_width.try_into().unwrap(),
                window_height.try_into().unwrap(),
            )
            .unwrap();
    }

    fn redraw(
        &mut self,
        display_components: &[Arc<Mutex<dyn DisplayComponent<Self::RenderingBackend>>>],
    ) {
        let window_dimensions = Vector2::new(
            self.window.inner_size().width,
            self.window.inner_size().height,
        );

        // Skip rendering if impossible window size
        if window_dimensions.min() == 0 {
            return;
        }

        let mut surface_buffer = self.surface.buffer_mut().unwrap();
        let mut surface_buffer_view = DMatrixViewMut::from_slice(
            bytemuck::cast_slice_mut(surface_buffer.as_mut()),
            window_dimensions.x as usize,
            window_dimensions.y as usize,
        );
        let display_component_guard = display_components[0].lock().unwrap();
        let display_component_buffer = display_component_guard.display_data();
        let display_component_buffer_size = Vector2::new(
            display_component_buffer.shape().0,
            display_component_buffer.shape().1,
        );

        // Clear the surface buffer
        surface_buffer_view.fill(Srgba::<u8>::new(0, 0, 0, 0xff));

        let scaling = window_dimensions
            .cast::<f32>()
            .component_div(&display_component_buffer_size.cast::<f32>());

        // Iterate over each pixel in the display component buffer
        for x in 0..display_component_buffer.nrows() {
            for y in 0..display_component_buffer.ncols() {
                let source_pixel = display_component_buffer[(x, y)];

                let dest_start = Vector2::new(x, y)
                    .cast::<f32>()
                    .component_mul(&scaling)
                    .map(f32::round)
                    .try_cast::<usize>()
                    .unwrap()
                    .zip_map(&window_dimensions, |dest_dim, window_dim| {
                        dest_dim.min(window_dim as usize)
                    });

                let dest_end = Vector2::new(x, y)
                    .cast::<f32>()
                    .add_scalar(1.0)
                    .component_mul(&scaling)
                    .map(f32::round)
                    .try_cast::<usize>()
                    .unwrap()
                    .zip_map(&window_dimensions, |dest_dim, window_dim| {
                        dest_dim.min(window_dim as usize)
                    });

                // Fill the destination pixels with the source pixel
                let mut destination_pixels = surface_buffer_view.view_mut(
                    (dest_start.x, dest_start.y),
                    (dest_end.x - dest_start.x, dest_end.y - dest_start.y),
                );

                destination_pixels.fill(source_pixel);
            }
        }

        drop(display_component_guard);
        surface_buffer.present().unwrap();
    }

    fn initialize_components(
        &mut self,
        components: &[Arc<Mutex<dyn DisplayComponent<Self::RenderingBackend>>>],
    ) {
        for component in components.iter() {
            component.lock().unwrap().initialize_display(());
        }
    }
}

impl WinitRenderBackendState for SoftwareState {
    fn new(window: Arc<Window>, global_config: Arc<RwLock<GlobalConfig>>) -> Self {
        let [window_width, window_height]: [u32; 2] = window.inner_size().into();

        let context = Context::new(window.clone()).unwrap();
        let mut surface = Surface::new(&context, window.clone()).unwrap();

        surface
            .resize(
                window_width.try_into().unwrap(),
                window_height.try_into().unwrap(),
            )
            .unwrap();

        Self {
            surface,
            window,
            egui_renderer: SoftwareEguiRenderer::default(),
            global_config,
        }
    }
}

pub struct SoftwareRendering;

impl RenderingBackend for SoftwareRendering {
    // Software rendering doesn't require any initialization data
    type ComponentInitializationData = ();
    type ComponentDisplayBuffer = DMatrix<Srgba<u8>>;
    type RuntimeState = SoftwareState;
}
