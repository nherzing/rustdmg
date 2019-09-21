use super::envelope::{Direction};

pub enum SweepAction {
    Disable,
    Nop,
    Update(u16)
}

#[derive(Debug)]
pub struct Sweep {
    enabled: bool,
    sweep_period: u8,
    direction: Direction,
    shift: u8,
    frequency: u16,
    periods_left: u8
}

impl Sweep {
    pub fn new_from_byte(byte: u8) -> Self {
        Sweep {
            enabled: false,
            sweep_period: (byte >> 4) & 0x7,
            direction:  if b3!(byte) == 0 { Direction::Increase } else { Direction::Decrease },
            shift: byte & 0x7,
            frequency: 0,
            periods_left: 0
        }
    }

    pub fn tick(&mut self) -> SweepAction {
        if !self.enabled || self.sweep_period == 0 { return SweepAction::Nop }

        if self.periods_left == 0 {
            self.periods_left = self.sweep_period;
            match self.new_frequency() {
                None => {
                    SweepAction::Disable

                }
                Some(f) => {
                    self.frequency = f;
                    match self.new_frequency() {
                        None => SweepAction::Disable,
                        Some(_) => SweepAction::Update(self.frequency)
                    }
                }
            }
        } else {
            self.periods_left -= 1;
            SweepAction::Nop
        }
    }

    pub fn trigger(&mut self, frequency: u16) -> SweepAction {
        self.frequency = frequency;
        self.periods_left = self.sweep_period;
        self.enabled = self.sweep_period != 0 || self.shift != 0;
        if self.enabled {
            match self.new_frequency() {
                None => SweepAction::Disable,
                Some(_) => SweepAction::Nop
            }
        } else {
            SweepAction::Nop
        }
    }

    fn new_frequency(&self) -> Option<u16> {
        let delta = self.frequency >> self.shift;
        match self.direction {
            Direction::Increase => {
                if self.frequency + delta > 2047 {
                    None
                } else {
                    Some(self.frequency + delta)
                }
            }
            Direction::Decrease => {
                Some(self.frequency - delta)
            }
        }
    }
}
