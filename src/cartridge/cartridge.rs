use std::str;
use std::fmt;
use std::fs;
use crate::memory::memory_map::{MemoryMappedDevice, MappedArea};
use super::Symbols;
use super::mbc::{build_mbc, Mbc, MbcType};

const ROM_BANK0_SIZE: usize = 0x4000;

#[derive(Debug)]
enum Mode {
    DMG,
    CGB,
    Both
}

pub struct Cartridge<> {
    path: std::path::PathBuf,
    data: Vec<u8>,
    rom_bank0: [u8; ROM_BANK0_SIZE],
    mbc: Box<Mbc>
}

impl Cartridge {
    pub fn new(path: std::path::PathBuf, save_path: Option<std::path::PathBuf>) -> Cartridge {
        let boot_rom = include_bytes!("cgb_boot.bin");
        let data = fs::read(path.clone()).unwrap();
        let mut rom_bank0 = [0; ROM_BANK0_SIZE];

        for i in 0..boot_rom.len() {
            rom_bank0[i] = boot_rom[i];
        }
        for i in 0x100..0x14F {
            rom_bank0[i] = data[i];
        }

        let mut mbc = build_mbc(data[0x147]);
        if let Some(save_path) = save_path {
            mbc.load_ram(&fs::read(save_path).unwrap());
        }

        Self {
            data, path, rom_bank0, mbc
        }
    }

    pub fn mapped_areas() -> [MappedArea; 3] {
        return [
            MappedArea(0xFF50, 0x1),
            MappedArea(0x0000, 0x8000),
            MappedArea(0xA000, 0x2000)
        ]
    }

    pub fn symbols(&self) -> Option<Symbols> {
        let mut symbol_path = self.path.clone();
        symbol_path.set_extension("sym");
        if symbol_path.exists() {
            Some(Symbols::new(symbol_path))
        } else {
            None
        }
    }

    pub fn clear_boot_rom(&mut self) {
        for i in 0..self.rom_bank0.len() {
            self.rom_bank0[i] = self.data[i];
        }
    }

    pub fn cgb_compatible(&self) -> bool {
        match self.cgb_support() {
            Mode::DMG => false,
            _ => true
        }
    }

    pub fn dump_ram(&self) -> Vec<u8> {
        self.mbc.dump_ram()
    }

    fn rom_bank1_start(&self) -> usize {
        (self.mbc.rom_bank_num() as usize * 0x4000) % self.data.len()
    }

    fn title(&self) -> &str {
        str::from_utf8(&self.data[0x134..=0x143]).unwrap_or("UNKNOWN")
    }

    fn mbc_type(&self) -> MbcType {
        self.mbc.mbc_type()
    }

    fn rom_size(&self) -> u8 {
        self.data[0x148]
    }

    fn ram_size(&self) -> u8 {
        self.data[0x149]
    }


    fn cgb_support(&self) -> Mode {
        match self.data[0x143] {
            0x80 => Mode::Both,
            0xC0 => Mode::CGB,
            _ => Mode::DMG
        }
    }
}

impl Clone for Cartridge {
    fn clone(&self) -> Self {
        let mut cartridge = Cartridge::new(self.path.clone(), None);
        cartridge.mbc.load_ram(&self.mbc.dump_ram());
        cartridge
    }
}

impl MemoryMappedDevice for Cartridge {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            0xFF50 => {
                self.clear_boot_rom();
            }
            _ => { self.mbc.set8(addr, byte) }
        }

    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            0xFF50 => 0,
            0x0000 ... 0x3FFF => self.rom_bank0[addr as usize],
            0x4000 ... 0x7FFF => self.data[self.rom_bank1_start() + addr as usize - 0x4000],
            0xA000 ... 0xBFFF => { self.mbc.get8(addr) }
            _ => { panic!("Can't read from Cartridge at 0x{:X}.", addr); }
        }
    }
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ title: {}, type: {:?}, rom_size: {}, ram_size: {}, cgb_support: {:?} }}",
               self.title(), self.mbc_type(),
               self.rom_size(), self.ram_size(),
               self.cgb_support()
        )
    }
}
