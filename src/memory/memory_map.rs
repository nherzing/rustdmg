use std::collections::HashMap;

const MEMORY_SIZE: usize = 0x10000;

pub struct MappedArea(pub u16, pub usize);

pub trait MemoryMappedDevice {
    fn set8(&mut self, addr: u16, byte: u8);
    fn get8(&self, addr: u16) -> u8;
    fn get_slice(&self, addr: u16, size: usize) -> &[u8];
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MemoryMappedDeviceId {
    ROMBank0,
    RAMBank0
}

pub struct MemoryMap {
    memory_map: [Option<MemoryMappedDeviceId>; MEMORY_SIZE]
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            memory_map: [None; MEMORY_SIZE]
        }
    }

    pub fn register(&mut self, id: MemoryMappedDeviceId, mapped_areas: &[MappedArea]) {
        for area in mapped_areas {
            let start = area.0 as usize;
            for i in start..(start as usize)+area.1 {
                self.memory_map[i] = Some(id);
            }
        }
    }

    pub fn get_id(&self, addr: u16) -> MemoryMappedDeviceId {
        match self.memory_map[addr as usize] {
            None => panic!("No device mapped for address 0x{:X}", addr),
            Some(id) => id
        }
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

    pub fn get(&mut self, id: MemoryMappedDeviceId) -> &mut MemoryMappedDevice {
        &mut **self.devices.get_mut(&id).expect("No device mapped")
    }
}
