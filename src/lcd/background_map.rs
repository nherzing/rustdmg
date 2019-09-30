use super::tiles::TileSet;

#[derive(Clone, Copy, Debug)]
enum VRamBank {
    Bank0,
    Bank1
}

#[derive(Clone, Copy, Debug)]
struct TileAttributes {
    palette_num: u8,
    vram_bank: VRamBank,
    x_flip: bool,
    y_flip: bool,
    above_oam: bool
}

impl TileAttributes {
    fn new(byte: u8) -> Self {
        TileAttributes {
            palette_num: byte & 0x7,
            vram_bank: if b3!(byte) == 0 { VRamBank::Bank0 } else { VRamBank::Bank1 },
            x_flip: b5!(byte) == 1,
            y_flip: b6!(byte) == 1,
            above_oam: b7!(byte) == 1
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BGPixel {
    pub value: u8,
    pub above_oam: bool,
    pub palette_number: u8
}

pub struct BackgroundMap<'a> {
    data: &'a [u8],
    tile_set0: TileSet<'a>,
    tile_set1: TileSet<'a>,
    tile_attributes: Option<Vec<TileAttributes>>
}

impl<'a> BackgroundMap<'a> {
    pub fn new(data: &'a [u8], tile_set0: TileSet<'a>, tile_set1: TileSet<'a>, attribute_data: Option<&'a [u8]>) -> BackgroundMap<'a> {
        BackgroundMap {
            data, tile_set0, tile_set1,
            tile_attributes: attribute_data.map(|ad| ad.iter().map(|d| TileAttributes::new(*d)).collect())
        }
    }

    pub fn row(&self, row: usize) -> [BGPixel; 256] {
        let mut result = [Default::default(); 256];
        for col_idx in 0..32 {
            let start = col_idx * 8;
            self.set_tile_row(row, col_idx, &mut result[start..start+8]);
        }
        result
    }

    fn set_tile_row(&self, row: usize, col_idx: usize, result: &mut [BGPixel]) {
        let idx = (row / 8) * 32 + col_idx;
        let tile_idx = self.data[idx];
        let tile_attributes = match &self.tile_attributes {
            None => TileAttributes::new(0),
            Some(ta) => ta[idx]
        };
        let tile = match tile_attributes.vram_bank {
            VRamBank::Bank0 => self.tile_set0.tile(tile_idx),
            VRamBank::Bank1 => self.tile_set1.tile(tile_idx)
        };
        let tile_row = tile.row(row % 8);
        for (idx, p) in result.iter_mut().enumerate() {
            *p = BGPixel {
                value: tile_row[idx],
                above_oam: tile_attributes.above_oam,
                palette_number: tile_attributes.palette_num
            };
        }
    }
}
