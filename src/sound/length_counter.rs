use crate::clocks::CLOCK_FREQ;

pub enum LengthCounterAction {
    Disable,
    Nop
}

use LengthCounterAction::*;

#[derive(Debug)]
pub struct LengthCounter {
    enabled: bool,
    length: u16,
    ticks_left: u32,
    max: u16
}

impl LengthCounter {
    pub fn new(max: u16) -> Self {
        LengthCounter {
            enabled: false,
            length: 0,
            ticks_left: 0,
            max
        }
    }

    pub fn tick(&mut self) -> LengthCounterAction {
        if !self.enabled || self.length == 0  { return Nop }

        if self.ticks_left == 0 {
            self.ticks_left = CLOCK_FREQ / 256;
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
                return Disable
            }
        } else {
            self.ticks_left -= 1;
        }
        Nop
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.ticks_left = CLOCK_FREQ / 256;
    }

    pub fn set_length(&mut self, length: u8) {
        self.length = self.max - (length as u16);
    }
}
