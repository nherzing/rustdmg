use std::iter;
use super::tiles::TileSet;
use crate::renderer::{GAME_WIDTH};

pub struct RowIterator<'a> {
    background_map: &'a BackgroundMap<'a>,
    row: usize,
    col: usize,
    current_tile_row: [u8; 8]
}

impl<'a> RowIterator<'a> {
    fn new(row: usize, background_map: &'a BackgroundMap) -> RowIterator<'a> {
        RowIterator {
            row,
            background_map,
            col: 0,
            current_tile_row: [0; 8]
        }
    }
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col >= GAME_WIDTH {
            None
        } else {
            let offset = self.col % 8;

            if offset == 0 {
                self.current_tile_row = self.background_map.tile_row(self.row, self.col);
            }
            self.col += 1;
            Some(self.current_tile_row[offset])
        }
    }
}

pub struct BackgroundMap<'a> {
    data: &'a [u8],
    tile_set: &'a TileSet<'a>
}

impl<'a> BackgroundMap<'a> {
    pub fn new(data: &'a [u8], tile_set: &'a TileSet) -> BackgroundMap<'a> {
        BackgroundMap { data, tile_set }
    }

    pub fn row_iter<'b>(&'b self, row: usize) -> RowIterator<'b> {
        RowIterator::new(row, &self)
    }

    fn tile_row(&self, row: usize, col: usize) -> [u8; 8] {
        let idx = (row / 8) * 32 + (col / 8);
        let tile = self.tile_set.tile(self.data[idx]);
        tile.row(row % 8)
    }
}
