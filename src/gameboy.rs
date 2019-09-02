use crate::memory_bus::{MemoryBus, EverythingDevice, MemoryMappedDeviceManager, MemoryMappedDeviceId, MemoryMap};
use crate::cpu::{Cpu};

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
