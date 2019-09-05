use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use crate::renderer::{Color, GAME_WIDTH, GAME_HEIGHT};
use crate::clocks::{CLOCKS_PER_SCREEN_REFRESH};
use super::tiles::TileSet;
use super::palette::Palette;
use super::background_map::BackgroundMap;

// macro_rules! b0 {
//     ($x:expr) => (($x >> 0) & 0x1);
// }

// macro_rules! b1 {
//     ($x:expr) => (($x >> 1) & 0x1);
// }

// macro_rules! b2 {
//     ($x:expr) => (($x >> 2) & 0x1);
// }

// macro_rules! b3 {
//     ($x:expr) => (($x >> 3) & 0x1);
// }

// macro_rules! b4 {
//     ($x:expr) => (($x >> 4) & 0x1);
// }

// macro_rules! b5 {
//     ($x:expr) => (($x >> 5) & 0x1);
// }

// macro_rules! b6 {
//     ($x:expr) => (($x >> 6) & 0x1);
// }

macro_rules! b7 {
    ($x:expr) => (($x >> 7) & 0x1);
}

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_SIZE: usize = 0x2000;
pub const VRAM_END: u16 = VRAM_START + VRAM_SIZE as u16 - 1;
const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const DMA: u16 = 0xFF46;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

const TILE_DATA_START: usize = 0x8000;
const TILE_DATA_OFFSET: usize = TILE_DATA_START - (VRAM_START as usize);
const TILE_DATA_SIZE: usize = 0x1800;

const TILE_MAP_START: usize = 0x9800;
const TILE_MAP_OFFSET: usize = TILE_MAP_START - (VRAM_START as usize);
const TILE_MAP_SIZE: usize = 0x400;

#[derive(Copy, Clone, Debug)]
enum Period {
    OAMSearch,
    PixelTransfer,
    HBlank,
    VBlank
}

impl Period {
    fn clocks(&self) -> u32 {
        match self {
            OAMSearch => 20,
            PixelTransfer => 43,
            HBlank => 51,
            VBlank => OAMSearch.clocks() + PixelTransfer.clocks() + HBlank.clocks()
        }
    }

    fn next(&self, ly: u8) -> (Period, u8) {
        match self {
            OAMSearch => (PixelTransfer, ly),
            PixelTransfer => (HBlank, ly),
            HBlank => {
                if ly < 144 {
                    (OAMSearch, ly+1)
                } else {
                    (VBlank, ly+1)
                }
            }
            VBlank => {
                if ly == 153 {
                    (OAMSearch, 0)
                } else {
                    (VBlank, ly+1)
                }
            }
        }
    }
}

use Period::*;

struct State {
    period: Period,
    clocks_left: u32,
    ly: u8
}

impl State {
    fn init() -> Self {
        State {
            period: OAMSearch,
            clocks_left: OAMSearch.clocks(),
            ly: 0
        }
    }

    fn tick(&mut self, clocks: u32) -> u32{
        if self.clocks_left < clocks {
            let rem_clocks = clocks - self.clocks_left;
            let (period, ly) = self.period.next(self.ly);
            self.period = period;
            self.ly = ly;
            self.clocks_left = period.clocks();
            rem_clocks
        } else {
            self.clocks_left -= clocks;
            0
        }
    }
}

pub struct LcdController {
    vram: [u8; VRAM_SIZE],
    frame_buffer: [Color; GAME_WIDTH * GAME_HEIGHT],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    clocks_since_render: u32,
    background_palette: Palette,
    state: State
}

impl LcdController {
    pub fn new() -> LcdController {
        LcdController {
            vram: [0; VRAM_SIZE],
            frame_buffer: [Color::Off; GAME_WIDTH * GAME_HEIGHT],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            clocks_since_render: 0,
            background_palette: Palette::new(0),
            state: State::init()
        }
    }

    pub fn mapped_areas() -> [MappedArea; 2] {
        [
            MappedArea(VRAM_START, VRAM_SIZE),
            MappedArea(LCDC, (WX - LCDC + 1) as usize)
        ]
    }

    pub fn frame_buffer(&self) -> &[Color; GAME_WIDTH * GAME_HEIGHT] {
        &self.frame_buffer
    }

    pub fn tick(&mut self, clocks: u32) {
        self.clocks_since_render += clocks;
        if b7!(self.lcdc) == 0 { return }

        let mut clocks_left = clocks;
        while clocks_left > 0 {
            clocks_left = self.state.tick(clocks);
            self.ly = self.state.ly;
        }
        if self.wants_refresh() {
            self.fill_framebuffer();
        }
    }

    fn fill_framebuffer(&mut self) {
        let bg_tile_data = &self.vram[TILE_DATA_OFFSET..TILE_DATA_OFFSET+TILE_DATA_SIZE];
        let bg_tile_set = TileSet::new(bg_tile_data);
        let bg_map_data = &self.vram[TILE_MAP_OFFSET..TILE_MAP_OFFSET+TILE_MAP_SIZE];
        let bg_map = BackgroundMap::new(bg_map_data, &bg_tile_set);
        for y in 0..GAME_HEIGHT {
            let shifted_y = (y + (self.scy as usize)) % 256;
            for (idx, p) in bg_map.row_iter(shifted_y).enumerate() {
                let color = self.background_palette.color(p);
                self.frame_buffer[y * GAME_WIDTH + idx] = color;
            }
        }
    }

    pub fn wants_refresh(&self) -> bool {
        self.clocks_since_render >= CLOCKS_PER_SCREEN_REFRESH
    }

    pub fn refresh(&mut self) {
        self.clocks_since_render = 0;
    }
}

impl MemoryMappedDevice for LcdController {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize] = byte;
            }
            BGP => {
                self.bgp = byte;
                self.background_palette = Palette::new(byte);
            }
            SCY => { self.scy = byte }
            LCDC => {
                if b7!(self.lcdc) == 0 && b7!(byte) == 1 {
                    for p in self.frame_buffer.iter_mut() {
                        *p = Color::White;
                    }
                }
                self.lcdc = byte;
            }
            _ => panic!("Invalid set address 0x{:X} mapped to LCD Controller", addr)

        }

    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize]
            }
            LY => { self.ly }
            SCY => { self.scy }
            _ => panic!("Invalid get address 0x{:X} mapped to LCD Controller", addr)

        }
    }

    fn get_slice(&self, _addr: u16, _size: usize) -> &[u8] {
        panic!("Can't obtain slice from LCD Controller")
    }
}
