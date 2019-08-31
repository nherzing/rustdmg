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

    fn load(&mut self, data: &[u8], start_at: usize) {
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

    pub fn read_bytes(&self, addr: u16, bytes: &mut [u8]) {
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = self.memory[addr as usize + i]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bytes() {
        let mb = MemoryBus::new_from_slice(&[1, 2, 3, 4, 5]);
        let mut bs: [u8; 3] = [0; 3];
        mb.read_bytes(2, &mut bs);
        assert_eq!(bs, [3, 4, 5])
    }
}
