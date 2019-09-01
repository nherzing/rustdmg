const MEMORY_SIZE: usize = 0x10000;

pub struct MemoryBus {
    memory: [u8; MEMORY_SIZE]
}

impl MemoryBus {
    pub fn new_from_slice(data: &[u8]) -> MemoryBus {
        let mut memory_bus = MemoryBus::new();
        memory_bus.load(data, 0);
        memory_bus
    }

    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; MEMORY_SIZE]
        }
    }

    pub fn load(&mut self, data: &[u8], start_at: usize) {
        for (i, &v) in data.iter().enumerate() {
            self.memory[start_at + i] = v
        }
    }

    pub fn set8(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    pub fn set16(&mut self, addr: u16, v: u16) {
        self.memory[addr as usize] = (v >> 8) as u8;
        self.memory[addr as usize + 1] = (v & 0xFF) as u8;
    }

    pub fn get8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn get16(&self, addr: u16) -> u16 {
        ((self.memory[addr as usize] as u16) << 8) + (self.memory[addr as usize + 1] as u16)
    }

    pub fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let idx = addr as usize;
        &self.memory[idx..idx+size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_slice() {
        let mb = MemoryBus::new_from_slice(&[1, 2, 3, 4, 5]);
        let bs = mb.get_slice(2, 3);
        assert_eq!(bs, [3, 4, 5])
    }
}
