use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use crate::renderer::{Color, GAME_WIDTH, GAME_HEIGHT};

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
            wx: 0
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
}

macro_rules! b0 {
    ($x:expr) => (($x >> 0) & 0x1);
}

macro_rules! b1 {
    ($x:expr) => (($x >> 1) & 0x1);
}

macro_rules! b2 {
    ($x:expr) => (($x >> 2) & 0x1);
}

macro_rules! b3 {
    ($x:expr) => (($x >> 3) & 0x1);
}

macro_rules! b4 {
    ($x:expr) => (($x >> 4) & 0x1);
}

macro_rules! b5 {
    ($x:expr) => (($x >> 5) & 0x1);
}

macro_rules! b6 {
    ($x:expr) => (($x >> 6) & 0x1);
}

macro_rules! b7 {
    ($x:expr) => (($x >> 7) & 0x1);
}


impl MemoryMappedDevice for LcdController {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            VRAM_START ... VRAM_END => {
                self.vram[(addr - VRAM_START) as usize] = byte;
            }
            BGP => { self.bgp = byte }
            SCY => { self.scy = byte }
            LCDC => {
                if b0!(self.lcdc) == 0 && b0!(byte) == 1 {
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
            },
            LY => {
                self.ly
            }

            _ => panic!("Invalid get address 0x{:X} mapped to LCD Controller", addr)

        }
    }

    fn get_slice(&self, _addr: u16, _size: usize) -> &[u8] {
        panic!("Can't obtain slice from LCD Controller")
    }
}
