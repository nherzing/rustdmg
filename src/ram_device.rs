use crate::memory::memory_map::{MemoryMappedDevice};

pub struct RamDevice {
    offset: usize,
    size: usize,
    memory: Vec<u8>,
    bank: usize
}

impl RamDevice {
    pub fn new(offset: usize, size: usize, banks: usize) -> RamDevice {
        let mut v = Vec::with_capacity(size * banks);
        v.resize(size * banks, 0);
        RamDevice {
            offset,
            size,
            memory: v,
            bank: 1
        }
    }

    fn bank_offset(&self) -> usize {
        (self.bank - 1) * self.size
    }
}

impl MemoryMappedDevice for RamDevice {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            0xFF70 => {
                self.bank = match byte & 0x3 {
                    0 | 1 => 1,
                    b => b as usize
                };
            }
            _ => {
                let mut idx = addr as usize - self.offset;
                if idx < self.size {
                    idx += self.bank_offset();
                    self.memory[idx] = byte;
                } else {
                    debug!("RAM write out of bounds {:X}", addr);
                }
            }
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            0xFF70 => self.bank as u8,
            _ => {
                let mut idx = addr as usize - self.offset;
                if idx < self.size {
                    idx += self.bank_offset();
                    self.memory[idx]
                } else {
                    debug!("RAM read out of bound {:X}", addr);
                    0xFF
                }
            }
        }
    }
}
