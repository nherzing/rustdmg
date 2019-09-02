use crate::memory_bus::{MemoryBus};
use self::registers::{Registers};

mod eval;
mod instr;
mod registers;

pub struct Cpu {
    registers: Registers,
    ime: bool,
    debug: bool
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            ime: false,
            debug: false
        }
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus) -> u8 {
        self.eval(memory_bus)
    }
}
