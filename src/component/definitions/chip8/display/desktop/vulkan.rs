use crate::{
    component::{
        definitions::chip8::display::{Chip8Display, Chip8DisplayImplementation, InternalState},
        display::DisplayComponent,
    },
    runtime::{desktop::display::vulkan::VulkanRendering, RenderingBackend},
};
use bitvec::{prelude::Msb0, view::BitView};
use nalgebra::{DMatrix, DMatrixViewMut, Point2};
use palette::Srgba;
use std::{ops::DerefMut, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, ClearColorImageInfo,
        CommandBufferUsage, CopyBufferToImageInfo, PrimaryCommandBufferAbstract,
    },
    device::Queue,
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    sync::GpuFuture,
};

pub struct VulkanState {
    pub staging_buffer: Subbuffer<[Srgba<u8>]>,
    pub render_image: Arc<Image>,
    pub queue: Arc<Queue>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl Chip8DisplayImplementation for VulkanState {
    fn draw_sprite(&mut self, position: Point2<u8>, sprite: &[u8]) -> bool {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        let mut staging_buffer = DMatrixViewMut::from_slice(staging_buffer.deref_mut(), 64, 32);

        let mut collided = false;

        for (y, sprite_row) in sprite.view_bits::<Msb0>().chunks(8).enumerate() {
            for (x, sprite_pixel) in sprite_row.iter().enumerate() {
                let x = position.x as usize + x;
                let y = position.y as usize + y;

                if x >= 64 || y >= 32 {
                    continue;
                }

                let old_sprite_pixel = staging_buffer[(x, y)] == Srgba::new(255, 255, 255, 255);

                if *sprite_pixel && old_sprite_pixel {
                    collided = true;
                }

                staging_buffer[(x, y)] = if *sprite_pixel ^ old_sprite_pixel {
                    Srgba::new(255, 255, 255, 255)
                } else {
                    Srgba::new(0, 0, 0, 255)
                };
            }
        }

        collided
    }

    fn clear_display(&mut self) {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        staging_buffer.fill(Srgba::new(0, 0, 0, 255));
    }

    fn get_display_buffer(&mut self) -> DMatrix<Srgba<u8>> {
        let staging_buffer = self.staging_buffer.read().unwrap();
        DMatrix::from_vec(64, 32, staging_buffer.to_vec())
    }

    fn set_screen_buffer(&mut self, buffer: DMatrix<Srgba<u8>>) {
        let mut staging_buffer = self.staging_buffer.write().unwrap();
        staging_buffer.copy_from_slice(buffer.as_slice());
    }

    fn commit_display(&mut self) {
        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        command_buffer
            // Clear the image
            .clear_color_image(ClearColorImageInfo::image(self.render_image.clone()))
            .unwrap()
            // Copy the staging buffer to the image
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                self.staging_buffer.clone(),
                self.render_image.clone(),
            ))
            .unwrap();
        command_buffer
            .build()
            .unwrap()
            .execute(self.queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }
}

impl DisplayComponent<VulkanRendering> for Chip8Display {
    fn initialize_display(
        &mut self,
        initialization_data: <VulkanRendering as RenderingBackend>::ComponentInitializationData,
    ) {
        let staging_buffer = Buffer::from_iter(
            initialization_data.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            vec![Srgba::new(0, 0, 0, 0); 64 * 32],
        )
        .unwrap();

        let render_image = Image::new(
            initialization_data.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent: [64, 32, 1],
                usage: ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        self.state = Some(InternalState::Vulkan(VulkanState {
            queue: initialization_data.queue,
            command_buffer_allocator: initialization_data.command_buffer_allocator,
            staging_buffer,
            render_image: render_image.clone(),
        }));
    }

    fn display_data(&self) -> &<VulkanRendering as RenderingBackend>::ComponentDisplayBuffer {
        let Some(InternalState::Vulkan(VulkanState { render_image, .. })) = self.state.as_ref()
        else {
            panic!("Display has not been initialized");
        };

        render_image
    }
}
