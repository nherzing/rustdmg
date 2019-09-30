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
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
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

#[derive(Clone, Copy)]
pub enum Mode {
    DMG,
    CGB
}

impl Mode {
    fn is_cgb(&self) -> bool {
        match self {
            Mode::DMG => false,
            Mode::CGB => true
        }
    }
}

pub struct Gameboy {
    cpu: Cpu,
    memory_map: MemoryMap,
    device_manager: MemoryMappedDeviceManager,
    mode: Mode
}

impl Gameboy {
    pub fn new(debug: bool, mode: Mode) -> Self {
        let mut cpu = Cpu::new(mode);
        if debug { cpu.enable_debug(); }

        let mut device_manager = MemoryMappedDeviceManager::new();
        match mode {
            Mode::CGB => {
                device_manager.set_ram_bank0(RamDevice::new(0xC000, 0x1000, 1));
                device_manager.set_ram_bank1(RamDevice::new(0xD000, 0x1000, 7));
            }
            Mode::DMG => {
                device_manager.set_ram_bank0(RamDevice::new(0xC000, 0x1000, 1));
                device_manager.set_ram_bank1(RamDevice::new(0xD000, 0x1000, 1));
            }
        }

        device_manager.set_interrupt_controller(InterruptController::new());
        device_manager.set_timer(TimerController::new());
        device_manager.set_lcd_controller(LcdController::new());
        device_manager.set_sound_controller(SoundController::new());
        device_manager.set_serial_controller(SerialController::new());
        device_manager.set_joypad_controller(JoypadController::new());
        device_manager.set_hram(RamDevice::new(0xFF80, 128, 1));

        Gameboy {
            cpu,
            memory_map: MemoryMap::new(),
            device_manager,
            mode
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
        self.memory_map.register(MemoryMappedDeviceId::Ignore, &[MappedArea(0x0000, 0x10000)]);

        self.memory_map.register(MemoryMappedDeviceId::RAMBank0, &[MappedArea(0xC000, 0x1000)]);
        self.memory_map.register(MemoryMappedDeviceId::RAMBank1, &[MappedArea(0xD000, 0x1000)]);
        if self.mode.is_cgb() {
            self.memory_map.register(MemoryMappedDeviceId::RAMBank1, &[MappedArea(0xFF70, 1)]);
        }

        self.memory_map.register(MemoryMappedDeviceId::Cartridge, &Cartridge::mapped_areas());
        self.memory_map.set_symbols(cartridge.symbols());
        self.device_manager.set_cartridge(cartridge);

        self.memory_map.register(MemoryMappedDeviceId::Interrupt, &InterruptController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Timer, &TimerController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::LCD, &LcdController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Sound, &SoundController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Joypad, &JoypadController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::Serial, &SerialController::mapped_areas());
        self.memory_map.register(MemoryMappedDeviceId::HRAM, &[MappedArea(0xFF80, 0xFFFF - 0xFF80)]);
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
