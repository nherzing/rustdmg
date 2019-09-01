use crate::memory_bus::{MemoryBus};
use crate::cpu::{Cpu};

pub struct Gameboy {
    cpu: Cpu
}

impl Gameboy {
    pub fn boot(cartridge: &[u8], debug: bool) {
        let boot_rom = include_bytes!("boot_rom.gb");
        let mut memory_bus = MemoryBus::new_from_slice(cartridge);
        memory_bus.load(boot_rom, 0);
        let mut cpu = Cpu::new(memory_bus);
        if debug { cpu.enable_debug(); }
        let mut gameboy = Gameboy { cpu };

        gameboy.go();
    }

    fn go(&mut self) {
        loop {
            self.tick();
        }
    }

    fn tick(&mut self) {
        self.cpu.step();
    }
}
