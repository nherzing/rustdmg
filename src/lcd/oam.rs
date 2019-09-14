use crate::renderer::{GAME_WIDTH};
use super::tiles::{TileSet, Tile};

#[derive(Clone, Copy)]
pub enum PaletteNumber {
    Zero,
    One
}

struct OamEntry<'a> {
    x: u8,
    y: u8,
    tile: Tile<'a>,
    above_background: bool,
    y_flip: bool,
    x_flip: bool,
    palette_number: PaletteNumber,
}

impl<'a> OamEntry<'a> {
    fn new(data: &[u8], tile_set: &'a TileSet) -> OamEntry<'a> {
        let flags = data[3];
        OamEntry {
            x: data[1],
            y: data[0],
            tile: tile_set.tile(data[2]),
            above_background: b7!(flags) == 0,
            y_flip: b6!(flags) == 1,
            x_flip: b5!(flags) == 1,
            palette_number: if b4!(flags) == 0 { PaletteNumber::Zero } else { PaletteNumber::One }
        }
    }

    fn is_on_row(&self, ly: u8) -> bool {
        ly + 16 >= self.y && ly + 16 < self.y + 8
    }

    fn row(&self, ly: u8) -> [u8; 8] {
        let mut tile_y = (ly + 16 - self.y) as usize;
        if self.y_flip {
            tile_y = 7 - tile_y;
        }
        let mut tile_row = self.tile.row(tile_y);
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
    entries: Vec<OamEntry<'a>>
}

impl<'a> OamEntries<'a> {
    pub fn new(oam: &'a [u8], tile_set: &'a TileSet) -> OamEntries<'a> {
        let entries = (0..40).map(|i| OamEntry::new(&oam[4*i..4*i+4], tile_set)).collect();
        OamEntries { entries }
    }

    pub fn row(&self, ly: u8) -> [Option<OamPixel>; GAME_WIDTH] {
        let mut result = [None; GAME_WIDTH];

        let visible_entries = self.entries.iter().filter(|e| e.is_on_row(ly)).take(10);
        for entry in visible_entries {
            let tile_row = entry.row(ly);
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
