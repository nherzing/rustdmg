use crate::memory::memory_bus::{MemoryBus};
use self::registers::{Registers, Register};

mod eval;
mod instr;
mod registers;

pub struct Cpu {
    registers: Registers,
    ime: bool,
    halted: bool,
    debug: bool
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            ime: false,
            halted: false,
            debug: false
        }
    }

    pub fn skip_boot_rom(&mut self) {
        self.registers.set16(Register::PC, 0x100);
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus) -> u32 {
        self.eval(memory_bus)
    }
}
