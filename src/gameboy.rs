use crate::memory::memory_bus::{MemoryBus};
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::{RamDevice};
use crate::rom_device::{RomDevice};
use crate::lcd_controller::{LcdController};
use crate::renderer::{Renderer};

use crate::cpu::{Cpu};

use MemoryMappedDeviceId::*;

const CLOCK_SPEED: u32 = 1024 * 1024;
const SCREEN_REFRESH: u32 = 17556;

pub struct Gameboy<'a> {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager,
    renderer: Renderer<'a>
}

impl<'a> Gameboy<'a> {
    pub fn boot(cartridge: &[u8], renderer: Renderer<'a>, debug: bool) -> Self {
        let mut cpu = Cpu::new();
        if debug { cpu.enable_debug(); }

        let device_manager = MemoryMappedDeviceManager::new(
            RomDevice::new(0x8000),
            RamDevice::new(0x8000, 0x8000),
            LcdController::new()
        );
        let mut gameboy = Gameboy {
            cpu,
            memory_map: MemoryMap::new(),
            device_manager,
            renderer
        };
        gameboy.map_devices(cartridge);


        gameboy.renderer.update_game(gameboy.device_manager.lcd_controller().frame_buffer());
        gameboy.renderer.refresh();

        gameboy
    }

    fn map_devices(&mut self, cartridge: &[u8]) {
        let boot_rom = include_bytes!("boot_rom.gb");
        self.memory_map.register(ROMBank0, &[MappedArea(0, 0x8000)]);

        let mut rom_bank = self.device_manager.rom_bank0();
        rom_bank.load(cartridge);
        rom_bank.load(boot_rom);

        let the_rest = self.device_manager.ram_bank0();
        self.memory_map.register(RAMBank0, &[MappedArea(0x8000, 0x8000)]);

        let lcd_controller = self.device_manager.lcd_controller();
        self.memory_map.register(LCD, &LcdController::mapped_areas());
    }

    pub fn tick(&mut self) {
        let mut mb = MemoryBus::new(&mut self.memory_map, &mut self.device_manager);

        let mut cycles = 0;
        while cycles < SCREEN_REFRESH {
            cycles += self.cpu.step(&mut mb);
        }

        self.renderer.update_game(self.device_manager.lcd_controller().frame_buffer());
        self.renderer.refresh();
    }
}
