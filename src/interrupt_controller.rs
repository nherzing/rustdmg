use crate::memory::memory_map::MemoryMappedDevice;
use crate::memory::memory_map::MappedArea;

const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;

#[derive(Debug)]
pub enum Interrupt {
    VBlank,
    Timer
}

impl Interrupt {
    fn addr(&self) -> u16 {
        match self {
            Interrupt::VBlank => 0x40,
            Interrupt::Timer => 0x50
        }
    }

    fn flag(&self) -> u8 {
        1 << match self {
            Interrupt::VBlank => 0,
            Interrupt::Timer => 2
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
        if self.enabled_and_requested(Interrupt::VBlank) {
            self.fire(Interrupt::VBlank, clear)
        } else if self.enabled_and_requested(Interrupt::Timer) {
            self.fire(Interrupt::Timer, clear)
        } else {
            None
        }
    }

    fn fire(&mut self, interrupt: Interrupt, clear: bool) -> Option<u16> {
        let flag = interrupt.flag();
        if clear { self.if_reg = self.if_reg ^ flag; }
        Some(interrupt.addr())
    }

    fn enabled_and_requested(&self, interrupt: Interrupt) -> bool {
        let flag = interrupt.flag();
        self.ie_reg & flag == flag && self.if_reg & flag == flag
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        self.if_reg = self.if_reg | interrupt.flag();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle() {
        let mut ic = InterruptController::new();

        assert_eq!(ic.handle(false), None);

        ic.set8(IE, 0xFF);
        assert_eq!(ic.handle(false), None);

        ic.set8(IF, 0xFF);
        assert_eq!(ic.handle(false), Some(0x50));
        assert_eq!(ic.get8(IF), 0xFF);

        assert_eq!(ic.handle(true), Some(0x50));
        assert_eq!(ic.get8(IF), 0xFB);
    }

    #[test]
    fn test_request() {
        let mut ic = InterruptController::new();

        ic.request(Interrupt::Timer);
        assert_eq!(ic.get8(IF), 0x04);
        ic.request(Interrupt::Timer);
        assert_eq!(ic.get8(IF), 0x04);
    }
}
