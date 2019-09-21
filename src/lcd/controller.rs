use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use crate::gameboy::Color;
use crate::renderer::{GAME_WIDTH};
use super::tiles::TileSet;
use super::palette::Palette;
use super::background_map::BackgroundMap;
use super::oam::{OamEntries, OamPixel, PaletteNumber, SpriteSize};
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

const STAT_RO_MASK: u8 = 0b111;
const STAT_RW_MASK: u8 = 0b01111000;

const TILE_MAP_0_START: usize = 0x9800;
const TILE_MAP_0_OFFSET: usize = TILE_MAP_0_START - (VRAM_START as usize);
const TILE_MAP_1_START: usize = 0x9C00;
const TILE_MAP_1_OFFSET: usize = TILE_MAP_1_START - (VRAM_START as usize);
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
    state: State
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
            state: State::init()
        }
    }

    pub fn mapped_areas() -> [MappedArea; 3] {
        [
            MappedArea(VRAM_START, VRAM_SIZE),
            MappedArea(LCDC, (LYC - LCDC + 1) as usize),
            MappedArea(BGP, (WX - BGP + 1) as usize)
        ]
    }

    pub fn tick<F>(&mut self, clocks: u32, frame_buffer: &mut [Color], mut fire_interrupt: F) where
    F: FnMut(Interrupt) {
        self.clocks_since_render += clocks;

        let orig_period = self.state.period;
        let mut clocks_left = clocks;
        while clocks_left > 0 {
            clocks_left = self.state.tick(clocks_left);
            self.ly = self.state.ly;
        }
        if orig_period != self.state.period {
            let ly_coincidence = self.ly == self.lyc;
            self.stat = (self.stat & STAT_RW_MASK) | ((ly_coincidence as u8) << 2) | self.state.period.mode();
            match self.state.period {
                OAMSearch => {
                    if b5!(self.stat) == 1 || (b6!(self.stat) == 1 && ly_coincidence) {
                        fire_interrupt(Interrupt::Stat);
                    }
                }
                PixelTransfer => {
                    self.fill_framebuffer(frame_buffer);
                }
                VBlank => {
                    fire_interrupt(Interrupt::VBlank);
                    if b4!(self.stat) == 1 || (b6!(self.stat) == 1 && ly_coincidence) {
                        fire_interrupt(Interrupt::Stat);
                    }
                }
                HBlank => {
                    if b3!(self.stat) == 1 {
                        fire_interrupt(Interrupt::Stat);
                    }
                }

            }
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

    fn window_enabled(&self) -> bool {
        b5!(self.lcdc) == 1
    }

    fn oam_enabled(&self) -> bool {
        b1!(self.lcdc) == 1
    }

    fn display_enabled(&self) -> bool {
        b7!(self.lcdc) == 1
    }

    fn map_data(&self, flag: u8) -> &[u8] {
        if flag == 1 {
            &self.vram[TILE_MAP_1_OFFSET..TILE_MAP_1_OFFSET+TILE_MAP_SIZE]
        } else {
            &self.vram[TILE_MAP_0_OFFSET..TILE_MAP_0_OFFSET+TILE_MAP_SIZE]
        }
    }

    fn bg_row(&self) -> [u8; GAME_WIDTH] {
        let bg_tile_set = self.bg_tile_set();
        let bg_map_data = self.map_data(b3!(self.lcdc));
        let bg_map = BackgroundMap::new(bg_map_data, &bg_tile_set);

        let bg_y = ((self.ly as usize) + (self.scy as usize)) % 256;

        let mut result = [0; GAME_WIDTH];
        let bg_row = bg_map.row(bg_y);
        for x in 0..GAME_WIDTH {
            result[x] = bg_row[(x + (self.scx as usize)) % 256];
        }
        result
    }

    fn window_row(&self) -> [Option<u8>; GAME_WIDTH] {
        if !self.window_enabled() ||
            self.wx as usize >= GAME_WIDTH + 7 ||
            self.ly < self.wy
        {
            return [None; GAME_WIDTH];
        }

        let tile_set = self.bg_tile_set();
        let map_data = self.map_data(b6!(self.lcdc));
        let map = BackgroundMap::new(map_data, &tile_set);
        let row = map.row((self.ly - self.wy) as usize);

        let mut result = [None; GAME_WIDTH];
        for x in 0..GAME_WIDTH {
            result[x] = if x+7 < (self.wx as usize) {
                None
            } else {
                Some(row[x + 7 - (self.wx as usize)])
            };
        }
        result
    }

    fn sprite_size(&self) -> SpriteSize {
        if b2!(self.lcdc) == 0 {
            SpriteSize::EightByEight
        } else {
            SpriteSize::EightBySixteen
        }
    }

    fn oam_row(&self) -> [Option<OamPixel>; GAME_WIDTH] {
        if !self.oam_enabled() {
            return [None; GAME_WIDTH];
        }

        let oam_tile_set = TileSet::new(&self.vram[0x0..0x1000], false);
        let oam_entries = OamEntries::new(&self.oam, &oam_tile_set, self.sprite_size());
        oam_entries.row(self.ly)
    }

    fn fill_framebuffer(&self, frame_buffer: &mut [Color]) {
        if !self.display_enabled() {
            for pixel in frame_buffer {
                *pixel = Color::Off
            }
            return
        }

        let bg_row = self.bg_row();
        let window_row = self.window_row();
        let oam_row = self.oam_row();
        let row_start = (self.ly as usize) * GAME_WIDTH;

        for x in 0..GAME_WIDTH {
            let bg_color = match window_row[x] {
                Some(p) => self.bg_palette.color(p),
                None => self.bg_palette.color(bg_row[x])
            };
            let color = match oam_row[x] {
                None => bg_color,
                Some(pixel) => {
                    if pixel.above_background || bg_row[x] == 0 {
                        match pixel.palette_number {
                            PaletteNumber::Zero => self.ob0_palette.color(pixel.value),
                            PaletteNumber::One => self.ob1_palette.color(pixel.value)
                        }
                    } else {
                        bg_color
                    }
                }
            };
            frame_buffer[row_start + x] = color
        }
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
            }
            STAT => {
               debug!("Set STAT: {:08b}", byte);
                self.stat = (self.stat & STAT_RO_MASK) | (byte & STAT_RW_MASK);
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
            SCX => {
                self.scx = byte;
            }
            LYC => {
                self.lyc = byte;
            }
            WX => {
                self.wx = byte;
            }
            WY => {
                self.wy = byte;
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
                self.stat
            }
            LY => {
                self.ly
            },
            SCX => self.scx,
            SCY => self.scy,
            LYC => self.lyc,
            BGP => self.bgp,
            OBP0 => self.obp0,
            OBP1 => self.obp1,
            WY => self.wy,
            WX => self.wx,
            _ => panic!("Invalid get address 0x{:X} mapped to LCD Controller", addr)

        }
    }
}
