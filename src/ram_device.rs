use crate::memory::memory_map::{MemoryMappedDevice};

pub struct RamDevice {
    offset: usize,
    memory: Vec<u8>
}

impl RamDevice {
    pub fn new(offset: usize, size: usize) -> RamDevice {
        let mut v = Vec::with_capacity(size);
        v.resize(size, 0);
        RamDevice {
            offset,
            memory: v
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &v) in data.iter().enumerate() {
            self.memory[i] = v
        }
    }
}

impl MemoryMappedDevice for RamDevice {
    fn set8(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize - self.offset] = byte;
    }

    fn get8(&self, addr: u16) -> u8 {
        self.memory[addr as usize - self.offset]
    }

    fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let idx = addr as usize - self.offset;
        &self.memory[idx..idx+size]
    }
}
