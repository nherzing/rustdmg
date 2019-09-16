use crate::memory::memory_bus::MemoryBus;
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::RamDevice;
use crate::interrupt_controller::{Interrupt, InterruptController};
use crate::joypad_controller::{JoypadController, JoypadInput};
use crate::timer_controller::TimerController;
use crate::lcd::LcdController;
use crate::sound::SoundController;
use crate::serial::SerialController;
use crate::renderer::{Renderer, Color, GAME_WIDTH, GAME_HEIGHT};
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

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
        device_manager.set_ram_bank0(RamDevice::new(0xC000, 0x4000));
        device_manager.set_interrupt_controller(InterruptController::new());
        device_manager.set_timer(TimerController::new());
        device_manager.set_lcd_controller(LcdController::new());
        device_manager.set_sound_controller(SoundController::new());
        device_manager.set_serial_controller(SerialController::new());
        device_manager.set_joypad_controller(JoypadController::new());

        Gameboy {
            cpu,
            memory_map: MemoryMap::new(),
            device_manager,
            renderer,
            frame_buffer: [Color::Off; GAME_WIDTH * GAME_HEIGHT],
        }
    }

    pub fn boot(&mut self, mut cartridge: Cartridge, skip_boot_rom: bool) {
        debug!("Booting: {:?}", cartridge);
        if skip_boot_rom {
            self.cpu.skip_boot_rom();
            cartridge.clear_boot_rom();
        }

        self.map_devices(cartridge);

        let lcd_controller = self.device_manager.lcd_controller();
        self.renderer.update_bg_tile_texture(lcd_controller.bg_tile_frame_buffer());
    }

    fn map_devices(&mut self, cartridge: Cartridge) {
        self.memory_map.register(MemoryMappedDeviceId::RAMBank0, &[MappedArea(0xC000, 0x4000)]);
        self.memory_map.register(MemoryMappedDeviceId::Cartridge, &Cartridge::mapped_areas());
        self.memory_map.set_symbols(cartridge.symbols());
        self.device_manager.set_cartridge(cartridge);

        self.memory_map.register(MemoryMappedDeviceId::Interrupt, &InterruptController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Timer, &TimerController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::LCD, &LcdController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Sound, &SoundController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Joypad, &JoypadController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Serial, &SerialController::mapped_areas());
    }

    pub fn tick(&mut self, pressed_inputs: &[JoypadInput]) {
        let mut mb = MemoryBus::new(&self.memory_map, &mut self.device_manager);

        mb.devices().joypad_controller().set_pressed(pressed_inputs);
        loop {
            let clocks = self.cpu.step(&mut mb);

            let mut interrupts = Vec::new();
            let mut fire_interrupt = |interrupt| interrupts.push(interrupt);
            mb.devices().timer().tick(clocks, &mut fire_interrupt);
            mb.devices().lcd_controller().tick(clocks, &mut self.frame_buffer, &mut fire_interrupt);
            mb.devices().serial_controller().tick(clocks, &mut fire_interrupt);
            let new_audio_data = mb.devices().sound_controller().tick(clocks);

            self.renderer.queue_audio(&new_audio_data);

            for interrupt in interrupts {
                mb.devices().interrupt_controller().request(interrupt);

                match interrupt {
                    Interrupt::VBlank => {
                        self.renderer.push_frame_buffer(&self.frame_buffer);
                        self.renderer.update_bg_tile_texture(mb.devices().lcd_controller().bg_tile_frame_buffer());
                        return
                    }
                    _ => {}
                }
            }
        }
    }
}
