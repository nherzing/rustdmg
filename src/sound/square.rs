use crate::clocks::CLOCK_FREQ;
use super::envelope::{VolumeEnvelope, Direction};

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0]
];

struct WaveGen {
    frequency: u16,
    timer: u16,
    duty_index: usize,
    cycle_offset: usize
}

impl WaveGen {
    fn new() -> Self {
        WaveGen {
            frequency: 0,
            timer: 0,
            duty_index: 0,
            cycle_offset: 0
        }
    }

    fn set_freq_lower(&mut self, data: u8) {
        self.frequency = (self.frequency & 0x700) | (data as u16);
    }

    fn set_freq_upper(&mut self, data: u8) {
        self.frequency = (self.frequency & 0xFF) | ((data as u16) << 8);
    }

    fn set_duty(&mut self, duty_index: u8) {
        self.duty_index = duty_index as usize;
    }

    fn tick(&mut self) {
        if self.timer == 0 {
            self.cycle_offset = (self.cycle_offset + 1) % 8;
            self.timer = 2048 - self.frequency;
        }
        self.timer -= 1;
    }

    fn sample(&self) -> u8 {
        DUTY_CYCLES[self.duty_index][self.cycle_offset]
    }
}

#[derive(Debug)]
pub struct Sweep {
    enabled: bool,
    sweep_period: u8,
    direction: Direction,
    shift: u8,
    frequency: u16,
    ticks_left: u32
}

enum SweepAction {
    Disable,
    Nop,
    Update(u16)
}

impl Sweep {
    pub fn new_from_byte(byte: u8) -> Self {
        Sweep {
            enabled: false,
            sweep_period: (byte >> 4) & 0x7,
            direction:  if b3!(byte) == 0 { Direction::Increase } else { Direction::Decrease },
            shift: byte & 0x7,
            frequency: 0,
            ticks_left: 0
        }
    }

    fn tick(&mut self) -> SweepAction {
        if !self.enabled || self.sweep_period == 0 { return SweepAction::Nop }

        if self.ticks_left == 0 {
            self.ticks_left = (CLOCK_FREQ / 128) * (self.sweep_period as u32);
            self.ticks_left -= 1;
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
            self.ticks_left -= 1;
            SweepAction::Nop
        }
    }

    fn trigger(&mut self, frequency: u16) -> SweepAction {
        self.frequency = frequency;
        self.ticks_left = (CLOCK_FREQ / 128) * (self.sweep_period as u32);
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

pub struct SquareWave {
    playing: bool,
    sweep: Sweep,
    square_wave: WaveGen,
    volume_envelope: VolumeEnvelope
}

impl SquareWave {
    pub fn new() -> Self {
        SquareWave {
            playing: false,
            sweep: Sweep::new_from_byte(0x80),
            square_wave: WaveGen::new(),
            volume_envelope: VolumeEnvelope::new_from_byte(0xF3)
        }
    }

    pub fn tick(&mut self) {
        if !self.playing {
            return
        }

        self.square_wave.tick();
        self.volume_envelope.tick();
        match self.sweep.tick() {
            SweepAction::Nop => {}
            SweepAction::Disable => {
                self.playing = false;
            }
            SweepAction::Update(frequency) => {
                self.square_wave.set_freq_lower(frequency as u8);
                self.square_wave.set_freq_upper((frequency >> 8) as u8 & 0x7);
            }
        }
    }

    pub fn sample(&self) -> u8 {
        if self.playing {
            self.volume_envelope.apply(self.square_wave.sample())
        } else {
            0
        }
    }

    pub fn set_sweep(&mut self, sweep: Sweep) {
        self.sweep = sweep;
    }

    pub fn set_duty(&mut self, duty_index: u8) {
        self.square_wave.set_duty(duty_index);
    }

    pub fn set_freq_lower(&mut self, data: u8) {
        self.square_wave.set_freq_lower(data);
    }

    pub fn set_freq_upper(&mut self, data: u8) {
        self.square_wave.set_freq_upper(data);
    }

    pub fn set_volume_envelope(&mut self, volume_envelope: VolumeEnvelope) {
        self.volume_envelope = volume_envelope;
    }

    pub fn restart(&mut self) {
        self.playing = true;
        match self.sweep.trigger(self.square_wave.frequency) {
            SweepAction::Disable => {
                self.playing = false
            }
            _ => { }
        }
    }

}
