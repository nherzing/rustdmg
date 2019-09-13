use crate::clocks::CLOCKS_PER_SERIAL_BIT_SHIFT;
use crate::memory::memory_map::{MemoryMappedDevice, MappedArea};
use crate::interrupt_controller::{Interrupt};

const SB: u16 = 0xFF01;
const SC: u16 = 0xFF02;

pub struct SerialController {
    sb: u8,
    sc: u8,
    clocks_to_shift: u32
}

impl SerialController {
    pub fn new() -> Self {
        SerialController {
            sb: 0,
            sc: 0,
            clocks_to_shift: 0
        }
    }

    pub fn mapped_areas() -> [MappedArea; 1] {
        [
            MappedArea(SB, 2)
        ]
    }

    pub fn tick<F>(&mut self, clocks: u32, mut fire_interrupt: F) where
        F: FnMut(Interrupt) {
        if b7!(self.sc) == 1 && b0!(self.sc) == 1 {
            if self.clocks_to_shift < clocks {
                self.sb = (self.sb << 1) | 0x1;
                self.sc = self.sc ^ (1 << 7);
                fire_interrupt(Interrupt::Serial);
            } else {
                self.clocks_to_shift -= clocks;
            }
        }
    }
}

impl MemoryMappedDevice for SerialController {
    fn get8(&self, addr: u16) -> u8 {
        match addr {
            SB => self.sb,
            SC => self.sc,
            _ => { panic!("Invalid get address 0x{:X} mapped to SerialController", addr) }
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            SB => {
                self.sb = byte;
            }
            SC => {
                if b7!(self.sc) == 0 && b7!(byte) == 1 {
                    self.clocks_to_shift = CLOCKS_PER_SERIAL_BIT_SHIFT;
                }
                self.sc = byte;
            }
            _ => { panic!("Invalid set address 0x{:X} mapped to SerialController", addr) }
        }
    }
}
