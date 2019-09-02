use crate::memory::memory_bus::{MemoryBus};
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::{RamDevice};
use crate::rom_device::{RomDevice};

use crate::cpu::{Cpu};

pub struct Gameboy {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager
}

impl Gameboy {
    pub fn boot(cartridge: &[u8], debug: bool) {
        let mut cpu = Cpu::new();
        if debug { cpu.enable_debug(); }
        let mut gameboy = Gameboy {
            cpu,
            memory_map: MemoryMap::new(),
            device_manager: MemoryMappedDeviceManager::new()
        };

        gameboy.map_devices(cartridge);
        gameboy.go();
    }

    fn map_devices(&mut self, cartridge: &[u8]) {
        let boot_rom = include_bytes!("boot_rom.gb");
        self.memory_map.register(MemoryMappedDeviceId::ROMBank0, &[MappedArea(0, 0x8000)]);

        let mut rom_bank = RomDevice::new(0x8000);
        rom_bank.load(cartridge);
        rom_bank.load(boot_rom);

        let the_rest = RamDevice::new(0x8000, 0x8000);
        self.memory_map.register(MemoryMappedDeviceId::RAMBank0, &[MappedArea(0x8000, 0x8000)]);

        self.device_manager.register(MemoryMappedDeviceId::ROMBank0, Box::new(rom_bank));
        self.device_manager.register(MemoryMappedDeviceId::RAMBank0, Box::new(the_rest));
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
