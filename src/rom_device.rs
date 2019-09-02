use crate::memory::memory_map::{MemoryMappedDevice};

pub struct RomDevice {
    memory: Vec<u8>
}

impl RomDevice {
    pub fn new(size: usize) -> RomDevice {
        let mut v = Vec::with_capacity(size);
        v.resize(size, 0);
        RomDevice {
            memory: v
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &v) in data.iter().enumerate() {
            self.memory[i] = v
        }
    }
}

impl MemoryMappedDevice for RomDevice {
    fn set8(&mut self, addr: u16, _byte: u8) {
        panic!("Can't write to ROM device at 0x{:X}.", addr);
    }

    fn get8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let idx = addr as usize;
        &self.memory[idx..idx+size]
    }
}
