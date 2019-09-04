use sdl2::pixels::{PixelFormatEnum, Color as PColor};
use sdl2::render::{WindowCanvas, TextureCreator, Texture};

const WHITE: PColor = PColor { r: 114, g: 129, b: 77, a: 255 };
const LIGHT_GRAY: PColor = PColor { r: 86, g: 107, b: 86, a: 255 };
const DARK_GRAY: PColor = PColor { r: 66, g: 92, b: 83, a: 255 };
const BLACK: PColor = PColor { r: 63, g: 82, b: 80, a: 255 };
const OFF: PColor = PColor { r: 140, g: 128, b: 47, a: 255 };
pub const GAME_WIDTH: usize = 160;
pub const GAME_HEIGHT: usize = 144;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
    Off
}

impl Color {
    fn to_pcolor(&self) -> PColor {
        match self {
            Color::White => WHITE,
            Color::LightGray => LIGHT_GRAY,
            Color::DarkGray => DARK_GRAY,
            Color::Black => BLACK,
            Color::Off => OFF
        }
    }
}

pub struct Renderer<'a> {
    canvas: &'a mut WindowCanvas,
    game_texture: Texture<'a>
}

impl<'a> Renderer<'a> {
    pub fn new(canvas: &'a mut WindowCanvas, texture_creator: &'a TextureCreator<sdl2::video::WindowContext>) -> Self {
        let game_texture = texture_creator.create_texture_streaming(
            PixelFormatEnum::RGB24, GAME_WIDTH as u32, GAME_HEIGHT as u32
        ).unwrap();

        Renderer { canvas, game_texture }
    }

    pub fn present(&mut self) {
        self.canvas.clear();
        self.canvas.present();
    }

    pub fn refresh(&mut self) {
        self.canvas.clear();
        self.canvas.copy(&self.game_texture, None, None);
        self.canvas.present();
    }

    pub fn update_game(&mut self, frame_buffer: &[Color]) {
        self.game_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..GAME_HEIGHT {
                for x in 0..GAME_WIDTH {
                    let offset = y*pitch + x*3;
                    let color = frame_buffer[y * GAME_WIDTH + x].to_pcolor();
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                }
            }
        });
    }
}
