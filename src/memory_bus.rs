use std::collections::HashMap;

const MEMORY_SIZE: usize = 0x10000;

pub struct MappedArea(pub u16, pub usize);

pub trait MemoryMappedDevice {
    fn mapped_areas(&self) -> Vec<MappedArea>;
    fn id(&self) -> MemoryMappedDeviceId;
    fn set8(&mut self, addr: u16, byte: u8);
    fn get8(&self, addr: u16) -> u8;
    fn get_slice(&self, addr: u16, size: usize) -> &[u8];
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MemoryMappedDeviceId {
    Everything
}

pub struct RomBank {
    start: u16,
    memory: [u8; MEMORY_SIZE]
}

impl RomBank {
    pub fn new(start: u16) -> RomBank {
        RomBank {
            start,
            memory: [0; MEMORY_SIZE]
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &v) in data.iter().enumerate() {
            self.memory[i] = v
        }
    }
}

impl MemoryMappedDevice for RomBank {
    fn mapped_areas(&self) -> Vec<MappedArea> {
        vec![MappedArea(self.start, 0x4000)]
    }

    fn id(&self) -> MemoryMappedDeviceId {
        MemoryMappedDeviceId::Everything
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    fn get8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let idx = addr as usize;
        &self.memory[idx..idx+size]
    }

}

pub struct MemoryMap {
    memory_map: [Option<MemoryMappedDeviceId>; MEMORY_SIZE]
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            memory_map: [Some(MemoryMappedDeviceId::Everything); MEMORY_SIZE]
        }
    }

    pub fn register(&mut self, device: &MemoryMappedDevice) {
        for area in device.mapped_areas() {
            let start = area.0 as usize;
            for i in start..(start as usize)+area.1 {
                self.memory_map[i] = Some(device.id());
            }
        }
    }

    pub fn get_id(&self, addr: u16) -> MemoryMappedDeviceId {
        self.memory_map[addr as usize].expect("No device mapped for address!")
    }
}

pub struct MemoryMappedDeviceManager {
    devices: HashMap<MemoryMappedDeviceId, Box<MemoryMappedDevice>>
}

impl MemoryMappedDeviceManager {
    pub fn new() -> MemoryMappedDeviceManager {
        MemoryMappedDeviceManager {
            devices: HashMap::new()
        }
    }

    pub fn register(&mut self, id: MemoryMappedDeviceId, device: Box<MemoryMappedDevice>) {
        self.devices.insert(id, device);
    }

    fn get(&mut self, id: MemoryMappedDeviceId) -> &mut MemoryMappedDevice {
        &mut **self.devices.get_mut(&id).expect("No device mapped")
    }
}


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

pub struct EverythingDevice {
    memory: [u8; 0x10000]
}

impl EverythingDevice {
    pub fn new(data: &[u8]) -> EverythingDevice {
        let mut memory = [0; 0x10000];
        for (i, &v) in data.iter().enumerate() {
            memory[i] = v
        }
        EverythingDevice { memory }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &v) in data.iter().enumerate() {
            self.memory[i] = v
        }
    }
}

impl MemoryMappedDevice for EverythingDevice {
    fn id(&self) -> MemoryMappedDeviceId {
        MemoryMappedDeviceId::Everything
    }

    fn mapped_areas(&self) -> Vec<MappedArea> {
        vec![MappedArea(0, 0x10000)]
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    fn get8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn get_slice(&self, addr: u16, size: usize) -> &[u8] {
        let idx = addr as usize;
        &self.memory[idx..idx+size]
    }
}

#[cfg(test)]
mod tests {
//    use super::*;

    // #[test]
    // fn test_get_slice() {
    //     let mb = MemoryBus::new_from_slice(&[1, 2, 3, 4, 5]);
    //     let bs = mb.get_slice(2, 3);
    //     assert_eq!(bs, [3, 4, 5])
    // }
}
