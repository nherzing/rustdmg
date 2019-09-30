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
    pub fn new() -> Self {
        Sweep {
            enabled: false,
            sweep_period: 0,
            direction: Direction::Increase,
            shift: 0,
            frequency: 0,
            periods_left: 0
        }
    }

    pub fn update(&mut self, byte: u8) {
        self.sweep_period = (byte >> 4) & 0x7;
        self.direction = if b3!(byte) == 0 { Direction::Increase } else { Direction::Decrease };
        self.shift = byte & 0x7;
    }

    pub fn tick(&mut self) -> SweepAction {
        if self.periods_left == 0 {
            self.periods_left = self.sweep_period;
        }

        if !self.enabled || self.sweep_period == 0 { return SweepAction::Nop }

        self.periods_left -= 1;
        if self.periods_left == 0 {
            self.periods_left = self.sweep_period;
            match self.new_frequency() {
                None => {
                    SweepAction::Disable
                }
                Some(f) => {
                    if self.shift != 0 {
                        self.frequency = f;
                    }

                    match self.new_frequency() {
                        None => SweepAction::Disable,
                        Some(_) => SweepAction::Update(self.frequency)
                    }
                }
            }
        } else {
            SweepAction::Nop
        }
    }

    pub fn trigger(&mut self, frequency: u16) -> SweepAction {
        self.frequency = frequency;
        self.periods_left = self.sweep_period;
        self.enabled = self.sweep_period != 0 || self.shift != 0;
        if self.shift != 0 {
            match self.new_frequency() {
                None => SweepAction::Disable,
                Some(_) => {
                    SweepAction::Update(self.frequency)
                }
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
