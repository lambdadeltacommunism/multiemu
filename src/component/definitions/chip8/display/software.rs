use crate::{
    component::{
        definitions::chip8::display::{Chip8Display, Chip8DisplayImplementation, InternalState},
        display::DisplayComponent,
    },
    runtime::{RenderingBackend, SoftwareRendering},
};
use bitvec::{prelude::Msb0, view::BitView};
use nalgebra::DMatrix;
use palette::Srgba;

pub struct SoftwareState {
    pub screen_buffer: DMatrix<Srgba<u8>>,
}

impl Chip8DisplayImplementation for SoftwareState {
    fn draw_sprite(&mut self, position: nalgebra::Point2<u8>, sprite: &[u8]) -> bool {
        let mut collided = false;

        for (y, sprite_row) in sprite.view_bits::<Msb0>().chunks(8).enumerate() {
            for (x, sprite_pixel) in sprite_row.iter().enumerate() {
                let x = position.x as usize + x;
                let y = position.y as usize + y;

                if x >= 64 || y >= 32 {
                    continue;
                }

                let old_sprite_pixel = self.screen_buffer[(x, y)] == Srgba::new(255, 255, 255, 255);

                if *sprite_pixel && old_sprite_pixel {
                    collided = true;
                }

                self.screen_buffer[(x, y)] = if *sprite_pixel ^ old_sprite_pixel {
                    Srgba::new(255, 255, 255, 255)
                } else {
                    Srgba::new(0, 0, 0, 255)
                };
            }
        }

        collided
    }

    fn clear_display(&mut self) {
        self.screen_buffer.fill(Srgba::new(0, 0, 0, 255));
    }

    fn get_display_buffer(&mut self) -> DMatrix<Srgba<u8>> {
        self.screen_buffer.clone()
    }

    fn set_screen_buffer(&mut self, buffer: DMatrix<Srgba<u8>>) {
        self.screen_buffer = buffer;
    }

    fn commit_display(&mut self) {
        // We don't use an extra staging buffer
    }
}

impl DisplayComponent<SoftwareRendering> for Chip8Display {
    fn initialize_display(
        &mut self,
        _initialization_data: <SoftwareRendering as RenderingBackend>::ComponentInitializationData,
    ) {
        let screen_buffer = DMatrix::from_element(64, 32, Srgba::new(0, 0, 0, 255));
        self.state = Some(InternalState::Software(SoftwareState { screen_buffer }));
    }

    fn display_data(&self) -> &<SoftwareRendering as RenderingBackend>::ComponentDisplayBuffer {
        let Some(InternalState::Software(SoftwareState { screen_buffer })) = self.state.as_ref()
        else {
            panic!("Display has not been initialized");
        };

        screen_buffer
    }
}
