use crate::memory::memory_map::MemoryMappedDevice;
use crate::memory::memory_map::MappedArea;

const P1: u16 = 0xFF00;

pub struct JoypadController {
    p1: u8
}

impl JoypadController {
    pub fn new() -> Self {
        JoypadController {
            p1: 0x0F
        }
    }

    pub fn mapped_areas() -> [MappedArea; 1] {
        [
            MappedArea(P1, 1)
        ]
    }
}

impl MemoryMappedDevice for JoypadController {
    fn get8(&self, addr: u16) -> u8 {
        match addr {
            P1 => self.p1,
            _ => { panic!("Invalid get address 0x{:X} mapped to JoypadController", addr) }
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            P1 => { self.p1 = (byte & 0x30) | (self.p1 & 0x0F) }
            _ => { panic!("Invalid set address 0x{:X} mapped to JoypadController", addr) }
        }
    }

}
