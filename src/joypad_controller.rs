use crate::memory::memory_map::MemoryMappedDevice;
use crate::memory::memory_map::MappedArea;

const P1: u16 = 0xFF00;

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

use JoypadInput::*;

pub struct JoypadController {
    p1: u8,
    direction_nib: u8,
    button_nib: u8
}

impl JoypadController {
    pub fn new() -> Self {
        JoypadController {
            p1: 0x3F,
            direction_nib: 0xF,
            button_nib: 0xF
        }
    }

    pub fn mapped_areas() -> [MappedArea; 1] {
        [
            MappedArea(P1, 1)
        ]
    }

    pub fn set_pressed(&mut self, pressed: &[JoypadInput]) {
        self.button_nib = 0x0;
        if !pressed.contains(&Start) {
            self.button_nib |= 1 << 3;
        }
        if !pressed.contains(&Select) {
            self.button_nib |= 1 << 2;
        }
        if !pressed.contains(&B) {
            self.button_nib |= 1 << 1;
        }
        if !pressed.contains(&A) {
            self.button_nib |= 1 << 0;
        }

        self.direction_nib = 0x0;
        if !pressed.contains(&Down) {
            self.direction_nib |= 1 << 3;
        }
        if !pressed.contains(&Up) {
            self.direction_nib |= 1 << 2;
        }
        if !pressed.contains(&Left) {
            self.direction_nib |= 1 << 1;
        }
        if !pressed.contains(&Right) {
            self.direction_nib |= 1 << 0;
        }
    }

    pub fn current_nib(&self) -> u8 {
        if b5!(self.p1) == 0 {
            self.button_nib
        } else if b4!(self.p1) == 0 {
            self.direction_nib
        } else {
            0xF
        }
    }
}

impl MemoryMappedDevice for JoypadController {
    fn get8(&self, addr: u16) -> u8 {
        match addr {
            P1 => {
                (0x30 & self.p1) | self.current_nib()
            }
            _ => { panic!("Invalid get address 0x{:X} mapped to JoypadController", addr) }
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            P1 => {
                self.p1 = byte & 0x30;
            }
            _ => { panic!("Invalid set address 0x{:X} mapped to JoypadController", addr) }
        }
    }

}
