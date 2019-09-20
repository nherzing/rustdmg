pub enum LengthCounterAction {
    Disable,
    Nop
}

use LengthCounterAction::*;

#[derive(Debug)]
pub struct LengthCounter {
    enabled: bool,
    length: u16,
    max: u16
}

impl LengthCounter {
    pub fn new(max: u16) -> Self {
        LengthCounter {
            enabled: false,
            length: 0,
            max
        }
    }

    pub fn tick(&mut self) -> LengthCounterAction {
        if !self.enabled || self.length == 0  { return Nop }

        debug!("LC: {} (pre dec)", self.length);
        self.length -= 1;
        if self.length == 0 {
            return Disable
        }
        Nop
    }

    pub fn set_enabled(&mut self, enabled: bool, next_is_length_counter: bool) -> LengthCounterAction {
        if !self.enabled && enabled && self.length > 0 && !next_is_length_counter {
            self.length -= 1;
            if self.length == 0 {
                return Disable
            }
        }
        self.enabled = enabled;
        Nop
    }

    pub fn set_length(&mut self, length: u8) {
        self.length = self.max - (length as u16);
    }

    pub fn trigger(&mut self) {
        if self.length == 0 {
            self.length = self.max;
        }
    }
}
