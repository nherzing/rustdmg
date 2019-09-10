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

    fn interrupt(&self) -> Option<Interrupt> {
        match self {
            VBlank => Some(Interrupt::VBlank),
            _ => None
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
    state: State,
    bg_tile_frame_buffer: [Color; 128 * 128]
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

    pub fn frame_buffer(&self) -> &[Color; GAME_WIDTH * GAME_HEIGHT] {
        &self.frame_buffer
    }

    pub fn bg_tile_frame_buffer(&self) -> &[Color; 128 * 128] {
        &self.bg_tile_frame_buffer
    }

    pub fn tick(&mut self, clocks: u32) -> Option<Interrupt> {
        self.clocks_since_render += clocks;
        if b7!(self.lcdc) == 0 { return None }

        let orig_period = self.state.period;
        let mut clocks_left = clocks;
        while clocks_left > 0 {
            clocks_left = self.state.tick(clocks_left);
            self.ly = self.state.ly;
        }
        if self.wants_refresh() {
            self.fill_framebuffer();
            self.fill_tile_framebuffer();
        }
        if orig_period != self.state.period {
            self.state.period.interrupt()
        } else {
            None
        }
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

    fn fill_framebuffer(&mut self) {
        let mut frame_buffer = [Color::Off; GAME_WIDTH * GAME_HEIGHT];
        let bg_tile_set = self.bg_tile_set();
        let bg_map_data = &self.vram[TILE_MAP_OFFSET..TILE_MAP_OFFSET+TILE_MAP_SIZE];
        let bg_map = BackgroundMap::new(bg_map_data, &bg_tile_set);
        for y in 0..GAME_HEIGHT {
            let shifted_y = (y + (self.scy as usize)) % 256;
            for (idx, p) in bg_map.row_iter(shifted_y).enumerate() {
                let color = self.background_palette.color(p);
                frame_buffer[y * GAME_WIDTH + idx] = color;
            }
        }
        self.frame_buffer = frame_buffer;
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
                    let color = self.background_palette.color(*p);
                    bg_tile_frame_buffer[row_start + k] = color;
                }
            }
        }

        self.bg_tile_frame_buffer = bg_tile_frame_buffer;
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
            LCDC => {
                if b7!(self.lcdc) == 0 && b7!(byte) == 1 {
                    for p in self.frame_buffer.iter_mut() {
                        *p = Color::White;
                    }
                }
                if b7!(self.lcdc) == 1 && b7!(byte) == 0 {
                    for p in self.frame_buffer.iter_mut() {
                        *p = Color::Off;
                    }
                }
                println!("LCDC: 0x{:b}", byte);
                self.lcdc = byte;
            }
            STAT => { println!("STAT: {:?}", byte); }
            BGP => {
                self.bgp = byte;
                self.background_palette = Palette::new(byte);
            }
            OBP0 => { }
            OBP1 => { }
            SCY => {
                println!("SCY: {}", byte);
                self.scy = byte;
            }
            SCX => {
                println!("SCX: {}", byte);
            }
            WY => { }
            WX => { }
            DMA => {
                println!("DMA");
            }
            _ => panic!("Invalid set address 0x{:X} mapped to LCD Controller", addr)

        }

    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize]
            }
            LCDC => 0, //self.lcdc,
            LY => {
                println!("READ LY: {}", self.ly);
                self.ly
            }
            SCY => self.scy,
            _ => panic!("Invalid get address 0x{:X} mapped to LCD Controller", addr)

        }
    }
}
