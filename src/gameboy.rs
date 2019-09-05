use std::{thread, time};
use crate::memory::memory_bus::{MemoryBus};
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::{RamDevice};
use crate::rom_device::{RomDevice};
use crate::lcd::{LcdController};
use crate::renderer::{Renderer};
use crate::clocks::{NS_PER_SCREEN_REFRESH};

use crate::cpu::{Cpu};

use MemoryMappedDeviceId::*;


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

        let mut device_manager = MemoryMappedDeviceManager::new();
        device_manager.set_rom_bank0(RomDevice::new(0x8000));
        device_manager.set_ram_bank0(RamDevice::new(0x8000, 0x8000));
        device_manager.set_lcd_controller(LcdController::new());

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

        let rom_bank = self.device_manager.rom_bank0();
        rom_bank.load(cartridge);
        rom_bank.load(boot_rom);

        self.memory_map.register(RAMBank0, &[MappedArea(0x8000, 0x8000)]);
        self.memory_map.register(LCD, &LcdController::mapped_areas());
    }

    pub fn tick(&mut self) {
        let ns_per_screen_refresh = time::Duration::from_nanos(NS_PER_SCREEN_REFRESH as u64);
        let now = time::Instant::now();
        let mut mb = MemoryBus::new(&mut self.memory_map, &mut self.device_manager);

        loop {
            let clocks = self.cpu.step(&mut mb);;

            let lcd_controller = mb.devices().lcd_controller();
            lcd_controller.tick(clocks);

            if lcd_controller.wants_refresh() {
                lcd_controller.refresh();
                self.renderer.update_game(lcd_controller.frame_buffer());
                self.renderer.refresh();

                let elapsed = now.elapsed();
                let to_wait = ns_per_screen_refresh.checked_sub(elapsed);
                match to_wait {
                    Some(d) => { thread::sleep(d) }
                    None => { println!("TOO SLOW: {:?}", elapsed) }
                }
                return
            }
        }
    }
}
