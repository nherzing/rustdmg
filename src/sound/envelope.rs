use crate::clocks::CLOCK_FREQ;

#[derive(Debug)]
enum Direction {
    Increase,
    Decrease
}

use Direction::*;

#[derive(Debug)]
pub struct VolumeEnvelope {
    volume: u8,
    direction: Direction,
    period: u8,
    ticks_left: u32
}

impl VolumeEnvelope {
    pub fn new_from_byte(byte: u8) -> Self {
        let volume = byte >> 4;
        let direction = if b3!(byte) == 0 { Decrease } else { Increase };
        let period = byte & 0x7;

        VolumeEnvelope {
            volume,
            direction,
            period,
            ticks_left: (period as u32) * CLOCK_FREQ / 64
        }
    }

    pub fn tick(&mut self) {
        if self.period == 0 { return }

        if self.ticks_left == 0 {
            self.ticks_left = (self.period as u32) * CLOCK_FREQ / 64;
            match self.direction {
                Decrease => if self.volume > 0 { self.volume -= 1; },
                Increase => if self.volume < 15 { self.volume += 1; }
            }
        }
        self.ticks_left -= 1;
    }

    pub fn apply(&self, input: u8) -> u8 {
        input * self.volume
    }
 }
