use std::{thread, time};
use crate::memory::memory_bus::MemoryBus;
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::RamDevice;
use crate::rom_device::RomDevice;
use crate::interrupt_controller::{Interrupt, InterruptController};
use crate::joypad_controller::{JoypadController, JoypadInput};
use crate::timer_controller::TimerController;
use crate::lcd::LcdController;
use crate::renderer::{Renderer, Color, GAME_WIDTH, GAME_HEIGHT};
use crate::clocks::{NS_PER_SCREEN_REFRESH};
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

use MemoryMappedDeviceId::*;

pub struct Gameboy<'a> {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager,
    renderer: Renderer<'a>,
    frame_buffer: [Color; GAME_WIDTH * GAME_HEIGHT]
}

impl<'a> Gameboy<'a> {
    pub fn new(renderer: Renderer<'a>, debug: bool) -> Self {
        let mut cpu = Cpu::new();
        if debug { cpu.enable_debug(); }

        let mut device_manager = MemoryMappedDeviceManager::new();
        device_manager.set_rom_bank0(RomDevice::new(0x8000));
        device_manager.set_ram_bank0(RamDevice::new(0x8000, 0x8000));
        device_manager.set_interrupt_controller(InterruptController::new());
        device_manager.set_timer(TimerController::new());
        device_manager.set_lcd_controller(LcdController::new());
        device_manager.set_joypad_controller(JoypadController::new());

        Gameboy {
            cpu,
            memory_map: MemoryMap::new(),
            device_manager,
            renderer,
            frame_buffer: [Color::Off; GAME_WIDTH * GAME_HEIGHT],
        }
    }

    pub fn boot(&mut self, cartridge: Cartridge, skip_boot_rom: bool) {
        debug!("Booting: {:?}", cartridge);
        if skip_boot_rom {
            self.cpu.skip_boot_rom();
        }
        self.map_devices(cartridge, skip_boot_rom);

        let lcd_controller = self.device_manager.lcd_controller();
        self.renderer.update_game(&self.frame_buffer);
        self.renderer.update_bg_tile_texture(lcd_controller.bg_tile_frame_buffer());
        self.renderer.refresh();
    }

    fn map_devices(&mut self, cartridge: Cartridge, skip_boot_rom: bool) {
        let boot_rom = include_bytes!("boot_rom.gb");
        self.memory_map.register(ROMBank0, &[MappedArea(0, 0x8000)]);
        self.memory_map.set_symbols(cartridge.symbols());

        let rom_bank = self.device_manager.rom_bank0();
        rom_bank.load_cartridge(&cartridge);
        if !skip_boot_rom { rom_bank.load(boot_rom); }

        self.memory_map.register(RAMBank0, &[MappedArea(0xC000, 0x4000)]);
        self.memory_map.register(Interrupt, &InterruptController::mapped_areas());
        self.memory_map.register(Timer, &TimerController::mapped_areas());
        self.memory_map.register(LCD, &LcdController::mapped_areas());
        self.memory_map.register(Joypad, &JoypadController::mapped_areas());
    }

    pub fn tick(&mut self, pressed_inputs: &[JoypadInput]) {
        let ns_per_screen_refresh = time::Duration::from_nanos(NS_PER_SCREEN_REFRESH as u64);
        let now = time::Instant::now();
        let mut mb = MemoryBus::new(&mut self.memory_map, &mut self.device_manager);

        mb.devices().joypad_controller().set_pressed(pressed_inputs);
        loop {
            let clocks = self.cpu.step(&mut mb);

            match mb.devices().timer().tick(clocks) {
                None => {}
                Some(interrupt) => {
                    mb.devices().interrupt_controller().request(interrupt)
                }
            }

            let lcd_interrupt = mb.devices().lcd_controller().tick(clocks, &mut self.frame_buffer);

            match lcd_interrupt {
                None => {}
                Some(interrupt) => {
                    mb.devices().interrupt_controller().request(interrupt)
                }
            }

            match lcd_interrupt {
                Some(Interrupt::VBlank) => {
                    self.renderer.update_game(&self.frame_buffer);
                    self.renderer.update_bg_tile_texture(mb.devices().lcd_controller().bg_tile_frame_buffer());

                    self.renderer.refresh();
                    let elapsed = now.elapsed();
                    let to_wait = ns_per_screen_refresh.checked_sub(elapsed);
                    match to_wait {
                        Some(d) => { thread::sleep(d) }
                        None => { debug!("TOO SLOW: {:?}", elapsed) }
                    }
                    return
                }
                _ => {}
            }
        }
    }
}
