use crate::renderer::{GAME_WIDTH};
use super::tiles::{TileSet};

#[derive(Clone, Copy)]
pub enum PaletteNumber {
    Zero,
    One
}

#[derive(Clone, Copy)]
pub enum SpriteSize {
    EightByEight,
    EightBySixteen
}

struct OamEntry {
    x: u8,
    y: u8,
    tile_num: u8,
    height: u8,
    above_background: bool,
    y_flip: bool,
    x_flip: bool,
    palette_number: PaletteNumber,
}

impl OamEntry {
    fn new(data: &[u8], size: SpriteSize) -> OamEntry {
        let height = match size {
            SpriteSize::EightByEight => 8,
            SpriteSize::EightBySixteen => 16
        };
        let flags = data[3];
        OamEntry {
            x: data[1],
            y: data[0],
            tile_num: if height == 8 { data[2] } else { data[2] & 0xFE },
            height,
            above_background: b7!(flags) == 0,
            y_flip: b6!(flags) == 1,
            x_flip: b5!(flags) == 1,
            palette_number: if b4!(flags) == 0 { PaletteNumber::Zero } else { PaletteNumber::One }
        }
    }

    fn is_on_row(&self, ly: u8) -> bool {
        ly + 16 >= self.y && ly + 16 < self.y + self.height
    }

    fn row(&self, ly: u8, tile_set: &TileSet) -> [u8; 8] {
        let mut tile_y = (ly + 16 - self.y) as usize;
        if self.y_flip {
            tile_y = (self.height as usize) - 1 - tile_y;
        }
        let mut tile_row = if tile_y < 8 {
            tile_set.tile(self.tile_num).row(tile_y)
        } else {
            tile_set.tile(self.tile_num + 1).row(tile_y % 8)
        };
        if self.x_flip {
            tile_row.reverse();
        }
        tile_row
    }
}

#[derive(Clone, Copy)]
pub struct OamPixel {
    pub value: u8,
    pub above_background: bool,
    pub palette_number: PaletteNumber
}

pub struct OamEntries<'a> {
    entries: Vec<OamEntry>,
    tile_set: &'a TileSet<'a>
}

impl<'a> OamEntries<'a> {
    pub fn new(oam: &'a [u8], tile_set: &'a TileSet, size: SpriteSize) -> OamEntries<'a> {
        let entries = (0..40).map(|i| OamEntry::new(&oam[4*i..4*i+4], size)).collect();
        OamEntries { entries, tile_set }
    }

    pub fn row(&self, ly: u8) -> [Option<OamPixel>; GAME_WIDTH] {
        let mut result = [None; GAME_WIDTH];

        let visible_entries = self.entries.iter().filter(|e| e.is_on_row(ly)).take(10);
        for entry in visible_entries {
            let tile_row = entry.row(ly, self.tile_set);
            let right_edge_x = entry.x as usize;
            let left_edge_x = (right_edge_x as i16) - 8;
            for idx in 0usize..8 {
                if left_edge_x + (idx as i16) < 0 { continue }
                if left_edge_x as usize + idx >= GAME_WIDTH { break }
                if tile_row[idx] == 0 { continue }
                if result[left_edge_x as usize + idx].is_none() {
                    result[left_edge_x as usize + idx] = Some(OamPixel {
                        value: tile_row[idx],
                        above_background: entry.above_background,
                        palette_number: entry.palette_number
                    });
                }
            }
        };

        result
    }
}
