use super::memory_map::{MemoryMap, MemoryMappedDeviceManager, MemoryMappedDevice};

pub struct MemoryBus<'a> {
    memory_map: &'a mut MemoryMap,
    resources: &'a mut MemoryMappedDeviceManager
}

impl<'a> MemoryBus<'a> {
    pub fn new(memory_map: &'a mut MemoryMap, resources: &'a mut MemoryMappedDeviceManager) -> MemoryBus<'a> {
        MemoryBus { memory_map, resources }
    }

    fn get_device(&mut self, addr: u16) -> &mut MemoryMappedDevice {
        self.resources.get(self.memory_map.get_id(addr))
    }

    pub fn set8(&mut self, addr: u16, byte: u8) {
        self.get_device(addr).set8(addr, byte);
    }

    pub fn set16(&mut self, addr: u16, v: u16) {
        let device = self.get_device(addr);
        device.set8(addr, (v >> 8) as u8);
        device.set8(addr + 1, (v & 0xFF) as u8);
    }

    pub fn get8(&mut self, addr: u16) -> u8 {
        self.get_device(addr).get8(addr)
    }

    pub fn get16(&mut self, addr: u16) -> u16 {
        let device = self.get_device(addr);
        ((device.get8(addr) as u16) << 8) + (device.get8(addr + 1) as u16)
    }

    pub fn get_slice(&mut self, addr: u16, size: usize) -> &[u8] {
        &self.get_device(addr).get_slice(addr, size)
    }
}
