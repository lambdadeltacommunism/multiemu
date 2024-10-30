use egui::FullOutput;
use std::sync::Arc;
use vulkano::{
    device::Device,
    format::Format,
    image::{Image, ImageCreateFlags, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    render_pass::RenderPass,
};
use winit::window::Window;

pub struct EguiRenderer {
    render_pass: Arc<RenderPass>,
    gui_render_image: Arc<Image>,
}

impl EguiRenderer {
    pub fn new(
        window: Arc<Window>,
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Self {
        let [window_width, window_height]: [u32; 2] = window.inner_size().into();

        let render_pass = vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                final_color: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [final_color],
                depth_stencil: {}
            }
        )
        .unwrap();

        let gui_render_image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                // For the srgb unorm conversion
                flags: ImageCreateFlags::MUTABLE_FORMAT,
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [window_width, window_height, 1],
                usage: ImageUsage::TRANSFER_DST
                    | ImageUsage::TRANSFER_SRC
                    | ImageUsage::COLOR_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        Self {
            render_pass,
            gui_render_image,
        }
    }

    pub fn redraw_egui(&self, egui_context: &egui::Context, full_output: FullOutput) {
        let vertexes = egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
    }
}
