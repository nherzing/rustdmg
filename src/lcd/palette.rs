use crate::gameboy::{Color};
use super::background_map::BGPixel;

const WHITE: Color = Color { r: 31, g: 31, b: 31 };
const LIGHT_GRAY: Color = Color { r: 20, g: 20, b: 20 };
const DARK_GRAY: Color = Color { r: 10, g: 10, b: 10 };
const BLACK: Color = Color { r: 0, g: 0, b: 0 };

pub struct Palette {
    palette: u8
}

impl Palette {
    pub fn new(palette: u8) -> Self {
        Palette { palette }
    }

    pub fn color(&self, byte: u8) -> Color {
        let shift = match byte {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 6,
            _ => panic!("Invalid color byte: {:X?}", byte)
        };
        let idx = (self.palette >> shift) & 0b11;
        match idx {
            0 => WHITE,
            1 => LIGHT_GRAY,
            2 => DARK_GRAY,
            3 => BLACK,
            _ => panic!("Invalid color idx: {}", idx)
        }
    }
}

pub struct PaletteManager {
    index: usize,
    auto_increment: bool,
    data: [u8; 64]
}

impl PaletteManager {
    pub fn new() -> Self {
        PaletteManager {
            index: 0,
            auto_increment: false,
            data: [0xFF; 64]
        }
    }

    pub fn set_index(&mut self, index: u8) {
        self.index = (index & 0x3F) as usize;
        self.auto_increment = b7!(index) == 1;
    }

    pub fn set8(&mut self, byte: u8) {
        self.data[self.index] = byte;
        if self.auto_increment {
            self.index += 1;
        }
    }

    pub fn color(&self, pixel: BGPixel) -> Color {
        let offset = (pixel.palette_number * 8 + pixel.value * 2) as usize;
        let b0 = self.data[offset] as u16;
        let b1 = self.data[offset + 1] as u16;
        let b10 = (b1 << 8) | b0;
        let r = b10 & 0x1F;
        let g = (b10 >> 5) & 0x1F;
        let b = (b10 >> 10) & 0x1F;
        Color::new(r as u8, g as u8, b as u8)
    }
}
