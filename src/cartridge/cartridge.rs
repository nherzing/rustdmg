use std::str;
use std::fmt;
use std::fs;
use crate::memory::memory_map::{MemoryMappedDevice, MappedArea};
use super::Symbols;

const ROM_BANK0_SIZE: usize = 0x4000;
const ROM_BANK1_SIZE: usize = 0x4000;
const ROM_BANKS_SIZE: usize = ROM_BANK0_SIZE + ROM_BANK1_SIZE;
const RAM_BANK0_SIZE: usize = 0x2000;

#[derive(Debug)]
enum MBCType {
    RomOnly = 0,
    MBC1 = 1,
    MBC1_RAM = 2,
    MBC1_RAM_BATTERY = 3,
    MBC2 = 5
}

pub struct Cartridge {
    path: std::path::PathBuf,
    data: Vec<u8>,
    rom_banks: [u8; ROM_BANK0_SIZE + ROM_BANK1_SIZE],
    ram_bank0: [u8; RAM_BANK0_SIZE]
}

impl Cartridge {
    pub fn new(path: std::path::PathBuf) -> Self {
        let boot_rom = include_bytes!("boot_rom.gb");
        let data = fs::read(path.clone()).unwrap();
        let mut rom_banks = [0; ROM_BANKS_SIZE];

        for i in 0..0x100 {
            rom_banks[i] = boot_rom[i];
        }
        for i in 0x100..ROM_BANKS_SIZE {
            rom_banks[i] = data[i];
        }

        Self {
            data, path, rom_banks,
            ram_bank0: [0; RAM_BANK0_SIZE]
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
        for i in 0..0x100 {
            self.rom_banks[i] = self.data[i];
        }
    }

    fn title(&self) -> &str {
        str::from_utf8(&self.data[0x134..=0x143]).unwrap_or("UNKNOWN")
    }

    fn mbc_type(&self) -> MBCType {
        match self.data[0x147] {
            0 => MBCType::RomOnly,
            1 => MBCType::MBC1,
            2 => MBCType::MBC1_RAM,
            3 => MBCType::MBC1_RAM_BATTERY,
            5 => MBCType::MBC2,
            x => panic!("Unknown MBCType {}", x)
        }
    }

    fn rom_size(&self) -> u8 {
        self.data[0x148]
    }

    fn ram_size(&self) -> u8 {
        self.data[0x149]
    }
}

impl MemoryMappedDevice for Cartridge {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            0xFF50 => {
                self.clear_boot_rom();
            },
            0xA000 ... 0xBFFF => {
                self.ram_bank0[addr as usize - 0xA000] = byte;
            }
            _ => { debug!("Can't write to Cartridge at 0x{:X} = 0x{:X}.", addr, byte); }
        }

    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ... 0x7FFF => self.rom_banks[addr as usize],
            0xA000 ... 0xBFFF => self.ram_bank0[addr as usize - 0xA000],
            _ => { panic!("Can't read from Cartridge at 0x{:X}.", addr); }
        }
    }

    fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let au = addr as usize;
        match addr {
            0x0000 ... 0x7FFF => &self.rom_banks[au..au+size],
            0xA000 ... 0xBfff => &self.ram_bank0[(au - 0xA000)..(au - 0xA000 + size)],
            _ => { panic!("Can't read slice from Cartridge at 0x:{:X}, len: {}", addr, size); }
        }
    }

}

impl fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ title: {}, type: {:?}, rom_size: {}, ram_size: {} }}",
               self.title(), self.mbc_type(),
               self.rom_size(), self.ram_size()
        )
    }
}
