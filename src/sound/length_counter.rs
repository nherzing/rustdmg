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
        debug!("SET ENABLED: was: {}, tobe: {}, len: {}", self.enabled, enabled, self.length);
        if !self.enabled && enabled && self.length > 0 && !next_is_length_counter {
            self.length -= 1;
            self.enabled = enabled;
            if self.length == 0 {
                self.enabled = false;
                return Disable
            }
        }
        self.enabled = enabled;
        Nop
    }

    pub fn set_length(&mut self, length: u8) {
        self.length = self.max - (length as u16);
        debug!("SET LENGTH: {}", self.length);
    }

    pub fn trigger(&mut self, will_enable: bool) {
        if self.length == 0 {
            self.length = self.max;
            if will_enable && !self.enabled {
                self.enabled = true;
                self.length -= 1;
            }
            debug!("RESET LENGTH TO {}, {}", self.max, self.length);
        }
    }
}
