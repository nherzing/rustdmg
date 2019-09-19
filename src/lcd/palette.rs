use crate::gameboy::{Color};

use Color::*;

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
            0 => White,
            1 => LightGray,
            2 => DarkGray,
            3 => Black,
            _ => panic!("Invalid color idx: {}", idx)
        }
    }
}
