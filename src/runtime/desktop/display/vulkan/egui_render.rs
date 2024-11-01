use bytemuck::{Pod, Zeroable};
use egui::{FullOutput, TextureId};
use nalgebra::Point2;
use palette::Srgba;
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferUsage,
    },
    descriptor_set::PersistentDescriptorSet,
    device::{Device, Queue},
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::{ImageView, ImageViewCreateInfo},
        Image, ImageCreateFlags, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::color_blend::{
        AttachmentBlend, BlendFactor, ColorBlendAttachmentState, ColorBlendState,
    },
    render_pass::RenderPass,
    swapchain::Swapchain,
    sync::GpuFuture,
    DeviceSize,
};
use winit::window::Window;


const VERTICES_PER_QUAD: DeviceSize = 4;
const VERTEX_BUFFER_SIZE: DeviceSize = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: DeviceSize = 1024 * 1024 * 2;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
struct EguiVertex {
    pos: Point2<f32>,
    uv: Point2<f32>,
    color: Srgba<f32>,
}

pub struct EguiRenderer {
    render_pass: Arc<RenderPass>,
    gui_render_image: Arc<Image>,
    gui_render_image_view_unorm: Arc<ImageView>,
    gui_render_image_view_srgb: Arc<ImageView>,
    texture_descriptors: HashMap<TextureId, Arc<PersistentDescriptorSet>>,
    texture_images: HashMap<TextureId, Arc<ImageView>>,
    queue: Arc<Queue>,
}

impl EguiRenderer {
    pub fn new(
        window: Arc<Window>,
        device: Arc<Device>,
        queue: Arc<Queue>,
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

        let gui_render_image_view_unorm = ImageView::new(
            gui_render_image.clone(),
            ImageViewCreateInfo {
                format: Format::R8G8B8A8_UNORM,
                ..ImageViewCreateInfo::from_image(&gui_render_image)
            },
        )
        .unwrap();

        let gui_render_image_view_srgb = ImageView::new(
            gui_render_image.clone(),
            ImageViewCreateInfo {
                format: Format::R8G8B8A8_SRGB,
                ..ImageViewCreateInfo::from_image(&gui_render_image)
            },
        )
        .unwrap();

        let vertex_index_buffer_pool = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                arena_size: INDEX_BUFFER_SIZE + VERTEX_BUFFER_SIZE,
                buffer_usage: BufferUsage::INDEX_BUFFER | BufferUsage::VERTEX_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let font_sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                // Egui textures cannot wrap at the edges at any cost
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                mipmap_mode: SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .unwrap();

        let pipeline = {
            let color_blend_state = ColorBlendState {
                attachments: vec![ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend {
                        src_color_blend_factor: BlendFactor::One,
                        src_alpha_blend_factor: BlendFactor::OneMinusConstantAlpha,
                        dst_alpha_blend_factor: BlendFactor::One,
                        ..AttachmentBlend::alpha()
                    }),
                    ..Default::default()
                }],
                ..ColorBlendState::default()
            };

            
        };

        Self {
            render_pass,
            gui_render_image,
            gui_render_image_view_unorm,
            gui_render_image_view_srgb,
            texture_descriptors: HashMap::new(),
            texture_images: HashMap::new(),
            queue,
        }
    }

    pub fn redraw_egui(
        &self,
        egui_context: &egui::Context,
        full_output: FullOutput,
        // Vulkaning rendering stuff
        previous_frame_future: &mut Option<Box<dyn GpuFuture>>,
        swapchain: Arc<Swapchain>,
    ) {
        let vertexes = egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
    }
}
