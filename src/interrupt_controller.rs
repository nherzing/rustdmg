use crate::memory::memory_map::MemoryMappedDevice;
use crate::memory::memory_map::MappedArea;

const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;

pub enum Interrupt {
    Timer
}

impl Interrupt {
    fn addr(&self) -> u16 {
        match self {
            Timer => 0x50
        }
    }

    fn requested(&self, if_reg: u8) -> bool {
        (if_reg & self.flag()) == self.flag()
    }

    fn enabled(&self, ie_reg: u8) -> bool {
        (ie_reg & self.flag()) == self.flag()
    }

    fn flag(&self) -> u8 {
        match self {
            Timer => 1 << 2
        }
    }
}

pub struct InterruptController {
    ie_reg: u8,
    if_reg: u8
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            ie_reg: 0,
            if_reg: 0
        }
    }

    pub fn mapped_areas() -> [MappedArea; 2] {
        [
            MappedArea(0xFFFF, 1),
            MappedArea(0xFF0F, 1)
        ]
    }

    pub fn handle(&mut self, clear: bool) -> Option<u16> {
        let flag = Interrupt::Timer.flag();
        if self.ie_reg & flag == flag && self.if_reg & flag == flag {
            if clear { self.if_reg = self.if_reg ^ flag; }
            Some(Interrupt::Timer.addr())
        } else {
            None
        }
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        self.if_reg = self.if_reg | interrupt.flag()
    }
}

impl MemoryMappedDevice for InterruptController {
    fn get8(&self, addr: u16) -> u8 {
        match addr {
            IE => self.ie_reg,
            IF => self.if_reg,
            _ => { panic!("Invalid get address 0x{:X} mapped to InterruptController", addr) }
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            IE => { self.ie_reg = byte }
            IF => { self.if_reg = byte }
            _ => { panic!("Invalid set address 0x{:X} mapped to InterruptController", addr) }
        }
    }
}
