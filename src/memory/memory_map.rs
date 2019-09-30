use crate::gameboy::Mode;
use crate::cartridge::Cartridge;
use crate::ram_device::RamDevice;
use crate::joypad_controller::JoypadController;
use crate::interrupt_controller::InterruptController;
use crate::timer_controller::TimerController;
use crate::serial::SerialController;
use crate::lcd::LcdController;
use crate::sound::SoundController;
use crate::cartridge::Symbols;

const MEMORY_SIZE: usize = 0x10000;

pub struct MappedArea(pub u16, pub usize);

pub trait MemoryMappedDevice {
    fn set8(&mut self, addr: u16, byte: u8);
    fn get8(&self, addr: u16) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum MemoryMappedDeviceId {
    Cartridge,
    RAMBank0,
    RAMBank1,
    Timer,
    Interrupt,
    Joypad,
    LCD,
    Serial,
    Sound,
    HRAM,
    Ignore
}

use MemoryMappedDeviceId::*;

pub struct MemoryMap {
    memory_map: [Option<MemoryMappedDeviceId>; MEMORY_SIZE],
    symbols: Option<Symbols>
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            memory_map: [None; MEMORY_SIZE],
            symbols: None
        }
    }

    pub fn set_symbols(&mut self, symbols: Option<Symbols>) {
        self.symbols = symbols;
    }

    pub fn get_sym(&self, addr: u16) -> Option<&String> {
        match &self.symbols {
            None => None,
            Some(c) => c.get(addr as usize)
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

struct NullDevice { }

impl NullDevice {
    fn new() -> Self {
        Self { }
    }
}

impl MemoryMappedDevice for NullDevice {
    fn set8(&mut self, _addr: u16, _byte: u8) {
        debug!("Unhandled write {:X}: {:X}", _addr, _byte);
    }

    fn get8(&self, _addr: u16) -> u8 {
        debug!("Unhandled get {:X}", _addr);
        0xFF
    }
}

pub struct MemoryMappedDeviceManager {
    cartridge: Option<Cartridge>,
    ram_bank0: Option<RamDevice>,
    ram_bank1: Option<RamDevice>,
    interrupt_controller: Option<InterruptController>,
    joypad_controller: Option<JoypadController>,
    timer: Option<TimerController>,
    lcd_controller: Option<LcdController>,
    sound_controller: Option<SoundController>,
    serial_controller: Option<SerialController>,
    hram: Option<RamDevice>,
    ignore: NullDevice
}

impl MemoryMappedDeviceManager {
    pub fn new() -> Self {
        MemoryMappedDeviceManager {
            cartridge: None,
            ram_bank0: None,
            ram_bank1: None,
            interrupt_controller: None,
            joypad_controller: None,
            timer: None,
            lcd_controller: None,
            sound_controller: None,
            serial_controller: None,
            hram: None,
            ignore: NullDevice::new()
        }
    }

    pub fn set_cartridge(&mut self, device: Cartridge) {
        if !device.cgb_compatible() {
            self.lcd_controller().set_mode(Mode::DMG);
        }
        self.cartridge = Some(device);
    }

    pub fn cartridge(&mut self) -> &mut Cartridge {
        match self.cartridge {
            Some(ref mut v) => v,
            None => panic!("No registered Cartridge")
        }
    }

    pub fn set_ram_bank0(&mut self, device: RamDevice) {
        self.ram_bank0 = Some(device);
    }

    pub fn ram_bank0(&mut self) -> &mut RamDevice {
        match self.ram_bank0 {
            Some(ref mut v) => v,
            None => panic!("No registered RAMBank0")
        }
    }

    pub fn set_ram_bank1(&mut self, device: RamDevice) {
        self.ram_bank1 = Some(device);
    }

    pub fn ram_bank1(&mut self) -> &mut RamDevice {
        match self.ram_bank1 {
            Some(ref mut v) => v,
            None => panic!("No registered RAMBank1")
        }
    }

    pub fn set_interrupt_controller(&mut self, device: InterruptController) {
        self.interrupt_controller = Some(device);
    }

    pub fn interrupt_controller(&mut self) -> &mut InterruptController {
        match self.interrupt_controller {
            Some(ref mut v) => v,
            None => panic!("No registered InteruptController")
        }
    }

    pub fn set_joypad_controller(&mut self, device: JoypadController) {
        self.joypad_controller = Some(device);
    }

    pub fn joypad_controller(&mut self) -> &mut JoypadController {
        match self.joypad_controller {
            Some(ref mut v) => v,
            None => panic!("No registered JoypadController")
        }
    }

    pub fn set_timer(&mut self, device: TimerController) {
        self.timer = Some(device);
    }

    pub fn timer(&mut self) -> &mut TimerController {
        match self.timer {
            Some(ref mut v) => v,
            None => panic!("No registered Timer")
        }
    }

    pub fn set_lcd_controller(&mut self, device: LcdController) {
        self.lcd_controller = Some(device);
    }

    pub fn lcd_controller(&mut self) -> &mut LcdController {
        match self.lcd_controller {
            Some(ref mut v) => v,
            None => panic!("No registered LCD")
        }
    }

    pub fn set_sound_controller(&mut self, device: SoundController) {
        self.sound_controller = Some(device);
    }

    pub fn sound_controller(&mut self) -> &mut SoundController {
        match self.sound_controller {
            Some(ref mut v) => v,
            None => panic!("No registered Sound Controller")
        }
    }

    pub fn set_serial_controller(&mut self, device: SerialController) {
        self.serial_controller = Some(device);
    }

    pub fn serial_controller(&mut self) -> &mut SerialController {
        match self.serial_controller {
            Some(ref mut v) => v,
            None => panic!("No registered SerialController")
        }
    }

    pub fn set_hram(&mut self, device: RamDevice) {
        self.hram = Some(device);
    }

    pub fn hram(&mut self) -> &mut RamDevice {
        match self.hram {
            Some(ref mut v) => v,
            None => panic!("No registered HRAM Device")
        }
    }

    pub fn get(&mut self, id: MemoryMappedDeviceId) -> &mut MemoryMappedDevice {
        match id {
            Cartridge => self.cartridge(),
            RAMBank0 => self.ram_bank0(),
            RAMBank1 => self.ram_bank1(),
            Timer => self.timer(),
            LCD => self.lcd_controller(),
            Sound => self.sound_controller(),
            Interrupt => self.interrupt_controller(),
            Joypad => self.joypad_controller(),
            Serial => self.serial_controller(),
            HRAM => self.hram(),
            Ignore => &mut self.ignore
        }
    }
}
