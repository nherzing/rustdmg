use crate::memory_bus::{MemoryBus};
use self::registers::{Registers};

mod eval;
mod instr;
mod registers;

struct Cpu {
    memory_bus: MemoryBus,
    registers: Registers,
    ime: bool
}

impl Cpu {
    pub fn new(memory_bus: MemoryBus) -> Cpu {
        Cpu {
            memory_bus,
            registers: Registers::new(),
            ime: false
        }
    }

    pub fn step(&mut self) -> u8 {
        self.eval()
    }
}
