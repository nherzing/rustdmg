use crate::memory_bus::{MemoryBus};
use self::registers::{Registers};

mod eval;
mod instr;
mod registers;

pub struct Cpu {
    memory_bus: MemoryBus,
    registers: Registers,
    ime: bool,
    debug: bool
}

impl Cpu {
    pub fn new(memory_bus: MemoryBus) -> Cpu {
        Cpu {
            memory_bus,
            registers: Registers::new(),
            ime: false,
            debug: false
        }
    }

    pub fn enable_debug(&mut self) {
        self.debug = true;
    }

    pub fn step(&mut self) -> u8 {
        self.eval()
    }
}
