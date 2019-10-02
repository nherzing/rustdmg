use crate::gameboy::{GAME_WIDTH, Mode};
use super::tiles::{TileSet};

#[derive(Clone, Copy)]
pub enum SpriteSize {
    EightByEight,
    EightBySixteen
}

#[derive(Clone, Copy, Debug)]
enum VRamBank {
    Bank0,
    Bank1
}

struct OamEntry {
    x: u8,
    y: u8,
    tile_num: u8,
    height: u8,
    above_background: bool,
    y_flip: bool,
    x_flip: bool,
    palette_number: u8,
    vram_bank: VRamBank
}

impl OamEntry {
    fn new(mode: Mode, data: &[u8], size: SpriteSize) -> OamEntry {
        let height = match size {
            SpriteSize::EightByEight => 8,
            SpriteSize::EightBySixteen => 16
        };
        let flags = data[3];
        OamEntry {
            x: data[1],
            y: data[0],
            vram_bank: if b3!(flags) == 0 { VRamBank::Bank0 } else { VRamBank::Bank1 },
            tile_num: if height == 8 { data[2] } else { data[2] & 0xFE },
            height,
            above_background: b7!(flags) == 0,
            y_flip: b6!(flags) == 1,
            x_flip: b5!(flags) == 1,
            palette_number: match mode {
                Mode::DMG => b4!(flags),
                Mode::CGB => flags & 0x7
            }
        }
    }

    fn is_on_row(&self, ly: u8) -> bool {
        ly + 16 >= self.y && ly + 16 < self.y + self.height
    }

    fn row(&self, ly: u8, tile_set0: &TileSet, tile_set1: &TileSet) -> [u8; 8] {
        let mut tile_y = (ly + 16 - self.y) as usize;
        let tile_set = match self.vram_bank {
            VRamBank::Bank0 => tile_set0,
            VRamBank::Bank1 => tile_set1
        };
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
    pub palette_number: u8
}

pub struct OamEntries<'a> {
    entries: Vec<OamEntry>,
    tile_set0: &'a TileSet<'a>,
    tile_set1: &'a TileSet<'a>
}

impl<'a> OamEntries<'a> {
    pub fn new(mode: Mode, oam: &'a [u8], tile_set0: &'a TileSet, tile_set1: &'a TileSet, size: SpriteSize) -> OamEntries<'a> {
        let entries = (0..40).map(|i| OamEntry::new(mode, &oam[4*i..4*i+4], size)).collect();
        OamEntries { entries, tile_set0, tile_set1 }
    }

    pub fn row(&self, ly: u8) -> [Option<OamPixel>; GAME_WIDTH] {
        let mut result = [None; GAME_WIDTH];

        let visible_entries = self.entries.iter()
            .filter(|e| e.is_on_row(ly))
            .take(10);
        for entry in visible_entries {
            let tile_row = entry.row(ly, self.tile_set0, self.tile_set1);
            let right_edge_x = entry.x as usize;
            let left_edge_x = (right_edge_x as i16) - 8;
            for idx in 0i16..8 {
                if left_edge_x + idx < 0 { continue }
                if left_edge_x + idx >= GAME_WIDTH as i16 { break }
                if tile_row[idx as usize] == 0 { continue }
                if result[(left_edge_x + idx) as usize].is_none() {
                    result[(left_edge_x + idx) as usize] = Some(OamPixel {
                        value: tile_row[idx as usize],
                        above_background: entry.above_background,
                        palette_number: entry.palette_number
                    });
                }
            }
        };

        result
    }
}
