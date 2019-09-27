use crate::memory::memory_bus::MemoryBus;
use crate::memory::memory_map::{MemoryMap, MappedArea, MemoryMappedDeviceManager, MemoryMappedDeviceId};
use crate::ram_device::RamDevice;
use crate::interrupt_controller::{Interrupt, InterruptController};
use crate::joypad_controller::{JoypadController};
use crate::timer_controller::TimerController;
use crate::lcd::LcdController;
use crate::sound::SoundController;
use crate::serial::SerialController;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

pub const GAME_WIDTH: usize = 160;
pub const GAME_HEIGHT: usize = 144;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
    Off
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoypadInput {
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
    A,
    B
}

pub struct Gameboy {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager
}

impl Gameboy {
    pub fn new(debug: bool) -> Self {
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
            device_manager
        }
    }

    pub fn boot(&mut self, mut cartridge: Cartridge, skip_boot_rom: bool) {
        debug!("Booting: {:?}", cartridge);
        if skip_boot_rom {
            self.cpu.skip_boot_rom();
            cartridge.clear_boot_rom();
        }

        self.map_devices(cartridge);
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

    pub fn tick(&mut self, pressed_inputs: &[JoypadInput], mut frame_buffer: &mut [Color], mut audio_queue: &mut Vec<f32>) {
        let mut mb = MemoryBus::new(&self.memory_map, &mut self.device_manager);
        let mut interrupts = Vec::with_capacity(10);

        mb.devices().joypad_controller().set_pressed(pressed_inputs);
        loop {
            let clocks = self.cpu.step(&mut mb);

            let mut fire_interrupt = |interrupt| interrupts.push(interrupt);
            mb.devices().timer().tick(clocks, &mut fire_interrupt);
            mb.devices().lcd_controller().tick(clocks, &mut frame_buffer, &mut fire_interrupt);
            mb.devices().serial_controller().tick(clocks, &mut fire_interrupt);
            mb.devices().sound_controller().tick(clocks, &mut audio_queue);

            for interrupt in &interrupts {
                mb.devices().interrupt_controller().request(*interrupt);

                match *interrupt {
                    Interrupt::VBlank => {
                        return
                    }
                    _ => {}
                }
            }
            interrupts.clear();
        }
    }

    pub fn fill_tile_framebuffer(&mut self, frame_buffer: &mut [Color]) {
        self.device_manager.lcd_controller().fill_tile_framebuffer(frame_buffer);
    }
}
