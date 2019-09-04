use crate::ram_device::{RamDevice};
use crate::rom_device::{RomDevice};
use crate::lcd_controller::{LcdController};

const MEMORY_SIZE: usize = 0x10000;

pub struct MappedArea(pub u16, pub usize);

pub trait MemoryMappedDevice {
    fn set8(&mut self, addr: u16, byte: u8);
    fn get8(&self, addr: u16) -> u8;
    fn get_slice(&self, addr: u16, size: usize) -> &[u8];
}

#[derive(Copy, Clone, Debug)]
pub enum MemoryMappedDeviceId {
    ROMBank0,
    RAMBank0,
    LCD
}

use MemoryMappedDeviceId::*;

pub struct MemoryMap {
    memory_map: [Option<MemoryMappedDeviceId>; MEMORY_SIZE]
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            memory_map: [None; MEMORY_SIZE]
        }
    }

    pub fn register(&mut self, id: MemoryMappedDeviceId, mapped_areas: &[MappedArea]) {
        for area in mapped_areas {
            let start = area.0 as usize;
            for i in start..(start as usize)+area.1 {
                self.memory_map[i] = Some(id);
            }
        }
    }

    pub fn get_id(&self, addr: u16) -> MemoryMappedDeviceId {
        match self.memory_map[addr as usize] {
            None => panic!("No device mapped for address 0x{:X}", addr),
            Some(id) => id
        }
    }
}

pub struct MemoryMappedDeviceManager {
    rom_bank0: RomDevice,
    ram_bank0: RamDevice,
    lcd_controller: LcdController,
}

impl MemoryMappedDeviceManager {
    pub fn new(rom_bank0: RomDevice, ram_bank0: RamDevice, lcd_controller: LcdController) -> Self {
        MemoryMappedDeviceManager {
            rom_bank0, ram_bank0, lcd_controller
        }
    }

    pub fn rom_bank0(&mut self) -> &mut RomDevice {
        &mut self.rom_bank0
    }

    pub fn ram_bank0(&mut self) -> &mut RamDevice {
        &mut self.ram_bank0
    }

    pub fn lcd_controller(&mut self) -> &mut LcdController {
        &mut self.lcd_controller
    }

    pub fn get(&mut self, id: MemoryMappedDeviceId) -> &mut MemoryMappedDevice {
        match id {
            ROMBank0 => &mut self.rom_bank0,
            RAMBank0 => &mut self.ram_bank0,
            LCD => &mut self.lcd_controller
        }
    }
}
