use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use crate::renderer::{Color, GAME_WIDTH, GAME_HEIGHT};
use crate::clocks::{CLOCKS_PER_SCREEN_REFRESH};
use super::tiles::TileSet;
use super::palette::Palette;
use super::background_map::BackgroundMap;
use crate::interrupt_controller::Interrupt;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_SIZE: usize = 0x2000;
pub const VRAM_END: u16 = VRAM_START + VRAM_SIZE as u16 - 1;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_SIZE: usize = 0xA0;
pub const OAM_END: u16 = OAM_START + OAM_SIZE as u16 - 1;
const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

const STAT_MODE_MASK: u8 = 0b111;
const STAT_RW_MASK: u8 = 0b01111000;

const TILE_MAP_START: usize = 0x9800;
const TILE_MAP_OFFSET: usize = TILE_MAP_START - (VRAM_START as usize);
const TILE_MAP_SIZE: usize = 0x400;

#[derive(Copy, Clone, Debug, PartialEq)]
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
                if ly < 143 {
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

    fn interrupt(&self) -> Option<Interrupt> {
        match self {
            VBlank => Some(Interrupt::VBlank),
            _ => None
        }
    }

    fn mode(&self) -> u8 {
        match self {
            OAMSearch => 2,
            PixelTransfer => 3,
            HBlank => 0,
            VBlank => 1
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

    fn tick(&mut self, clocks: u32) -> u32 {
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
    oam: [u8; OAM_SIZE],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    clocks_since_render: u32,
    bg_palette: Palette,
    ob0_palette: Palette,
    ob1_palette: Palette,
    state: State,
    bg_tile_frame_buffer: [Color; 128 * 128]
}

impl LcdController {
    pub fn new() -> LcdController {
        LcdController {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            clocks_since_render: 0,
            bg_palette: Palette::new(0),
            ob0_palette: Palette::new(0),
            ob1_palette: Palette::new(0),
            state: State::init(),
            bg_tile_frame_buffer: [Color::Off; 128 * 128],
        }
    }

    pub fn mapped_areas() -> [MappedArea; 2] {
        [
            MappedArea(VRAM_START, VRAM_SIZE),
            MappedArea(LCDC, (WX - LCDC + 1) as usize)
        ]
    }

    pub fn bg_tile_frame_buffer(&mut self) -> &[Color; 128 * 128] {
        self.fill_tile_framebuffer();
        &self.bg_tile_frame_buffer
    }

    pub fn tick(&mut self, clocks: u32, frame_buffer: &mut [Color]) -> Option<Interrupt> {
        self.clocks_since_render += clocks;

        let orig_period = self.state.period;
        let mut clocks_left = clocks;
        while clocks_left > 0 {
            clocks_left = self.state.tick(clocks_left);
            self.ly = self.state.ly;
        }
        if orig_period != self.state.period {
            self.stat = (self.stat & STAT_RW_MASK) | self.state.period.mode();
            if self.state.period == PixelTransfer {
                self.fill_framebuffer(frame_buffer);
            }
            self.state.period.interrupt()
        } else {
            None
        }
    }

    pub fn dma(&mut self, data: &[u8]) {
        self.oam.copy_from_slice(data);
    }

    fn bg_tile_set(&self) -> TileSet {
        if b4!(self.lcdc) == 0 {
            let bg_tile_data = &self.vram[0x800..0x1800];
            TileSet::new(bg_tile_data, true)
        } else {
            let bg_tile_data = &self.vram[0x0..0x1000];
            TileSet::new(bg_tile_data, false)
        }
    }

    fn fill_framebuffer(&self, frame_buffer: &mut [Color]) {
        let bg_tile_set = self.bg_tile_set();
        let bg_map_data = &self.vram[TILE_MAP_OFFSET..TILE_MAP_OFFSET+TILE_MAP_SIZE];
        let bg_map = BackgroundMap::new(bg_map_data, &bg_tile_set);

        let row_start = (self.ly as usize) * GAME_WIDTH;
        let bg_y = ((self.ly as usize) + (self.scy as usize)) % 256;
        let bg_row = bg_map.row(bg_y);
        for x in 0..GAME_WIDTH {
            let color = self.bg_palette.color(bg_row[x]);
            frame_buffer[row_start + x] = color
        }
    }

    fn fill_tile_framebuffer(&mut self) {
        let mut bg_tile_frame_buffer =  [Color::Off; 128 * 128];
        let bg_tile_set = self.bg_tile_set();

        for i in 0usize..256 {
            let tile = bg_tile_set.tile(i as u8);
            let origin = (i / 16)*128*8 + (i % 16)*8;
            for j in 0..8 {
                let row = tile.row(j);
                let row_start = origin + 128 * j;
                for (k, p) in row.iter().enumerate() {
                    let color = self.bg_palette.color(*p);
                    bg_tile_frame_buffer[row_start + k] = color;
                }
            }
        }

        self.bg_tile_frame_buffer = bg_tile_frame_buffer;
    }
}

impl MemoryMappedDevice for LcdController {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize] = byte;
            }
            OAM_START ... OAM_END => {
                self.oam[(addr - OAM_START) as usize] = byte;
            }
            LCDC => {
                self.lcdc = byte;
               debug!("Set LCDC: {:08b}", byte);
            }
            STAT => {
               debug!("Set STAT: {:08b}", byte);
                self.stat = (self.stat & STAT_MODE_MASK) | (byte & STAT_RW_MASK);
            }
            BGP => {
                self.bgp = byte;
                self.bg_palette = Palette::new(byte);
            }
            OBP0 => {
                self.obp0 = byte;
                self.ob0_palette = Palette::new(byte);
            }
            OBP1 => {
                self.obp1 = byte;
                self.ob1_palette = Palette::new(byte);
            }
            SCY => {
                self.scy = byte;
            }
            SCX => { }
            WY => { }
            WX => { }
            DMA => {
               debug!("Set DMA: {:08b}", byte);
            }
            _ => panic!("Invalid set address 0x{:X} mapped to LCD Controller", addr)

        }

    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize]
             }
            LCDC => self.lcdc,
            STAT => {
               debug!("Read STAT: {:08b}", self.stat);
                self.stat
            }
            LY => {
                self.ly
            }
            SCY => self.scy,
            _ => panic!("Invalid get address 0x{:X} mapped to LCD Controller", addr)

        }
    }
}
