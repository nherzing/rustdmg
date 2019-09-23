use super::tiles::TileSet;

pub struct BackgroundMap<'a> {
    data: &'a [u8],
    tile_set: TileSet<'a>
}

impl<'a> BackgroundMap<'a> {
    pub fn new(data: &'a [u8], tile_set: TileSet<'a>) -> BackgroundMap<'a> {
        BackgroundMap { data, tile_set }
    }

    pub fn row(&self, row: usize) -> [u8; 256] {
        let mut result = [0; 256];
        for col_idx in 0..32 {
            let tile_row = self.tile_row(row, col_idx);
            for offset in 0..8 {
                result[col_idx * 8 + offset] = tile_row[offset];
            }
        }
        result
    }

    fn tile_row(&self, row: usize, col_idx: usize) -> [u8; 8] {
        let idx = (row / 8) * 32 + col_idx;
        let tile = self.tile_set.tile(self.data[idx]);
        tile.row(row % 8)
    }
}
