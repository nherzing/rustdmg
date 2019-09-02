use crate::memory::memory_bus::{MemoryBus};
use crate::memory::memory_map::{MemoryMap, MemoryMappedDevice, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};

use crate::cpu::{Cpu};

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


pub struct Gameboy {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager
}

impl Gameboy {
    pub fn boot(cartridge: &[u8], debug: bool) {
        let boot_rom = include_bytes!("boot_rom.gb");
        let mut device = EverythingDevice::new(boot_rom);
        device.load(cartridge);
        device.load(boot_rom);
        let mut mm = MemoryMap::new();
        mm.register(&device);
        let mut mmdm = MemoryMappedDeviceManager::new();
        mmdm.register(MemoryMappedDeviceId::Everything, Box::new(device));
        let mut cpu = Cpu::new();
        if debug { cpu.enable_debug(); }
        let mut gameboy = Gameboy {
            cpu,
            memory_map: mm,
            device_manager: mmdm
        };

        gameboy.go();
    }

    fn go(&mut self) {
        loop {
            self.tick();
        }
    }

    fn tick(&mut self) {
        let mut mb = MemoryBus::new(&mut self.memory_map, &mut self.device_manager);
        self.cpu.step(&mut mb);
    }
}
