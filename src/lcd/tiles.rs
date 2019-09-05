pub struct Tile<'a> {
    data: &'a [u8]
}

impl <'a> Tile<'a> {
    pub fn new(data: &'a [u8]) -> Tile<'a> {
        Tile { data }
    }
    pub fn row(&self, row: usize) -> [u8; 8] {
        let lb = self.data[row * 2];
        let ub = self.data[row * 2 + 1];
        let mut result = [0; 8];
        for i in 0..8 {
            let shift = 7 - i;
            result[i] = ((ub >> shift << 1) & 0b10) | ((lb >> shift) & 0b01);
        }
        result
    }
}

pub struct TileSet<'a> {
    data: &'a [u8]
}

impl<'a> TileSet<'a> {
    pub fn new(data: &'a [u8]) -> TileSet<'a> {
        TileSet { data }
    }

    pub fn tile(&self, idx: u8) -> Tile {
        let start = (idx as usize) * 16;
        Tile::new(&self.data[start..start+16])
    }
}
