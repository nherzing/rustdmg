use super::frame_sequencer::FrameSequencer;
use super::envelope::{VolumeEnvelope};
use super::sweep::{Sweep, SweepAction};
use super::length_counter::{LengthCounter, LengthCounterAction};

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

pub struct SquareWave {
    dac_on: bool,
    enabled: bool,
    sweep: Sweep,
    square_wave: WaveGen,
    volume_envelope: VolumeEnvelope,
    length_counter: LengthCounter
}

impl SquareWave {
    pub fn new() -> Self {
        let mut sweep = Sweep::new();
        sweep.update(0x80);
        SquareWave {
            dac_on: false,
            enabled: false,
            sweep,
            square_wave: WaveGen::new(),
            volume_envelope: VolumeEnvelope::new_from_byte(0xF3),
            length_counter: LengthCounter::new(64)
        }
    }

    pub fn is_on(&self) -> bool {
        self.dac_on && self.enabled
    }

    pub fn tick(&mut self, fs: &FrameSequencer) {
        if fs.is_length_counter() {
            match self.length_counter.tick() {
                LengthCounterAction::Nop => {}
                LengthCounterAction::Disable => {
                    self.enabled = false;
                    return
                }
            }
        }
        if !self.dac_on || !self.enabled {
            return
        }

        if fs.is_sweep() {
            match self.sweep.tick() {
                SweepAction::Nop => {}
                SweepAction::Disable => {
                    self.enabled = false;
                    return
                }
                SweepAction::Update(frequency) => {
                    self.square_wave.set_freq_lower(frequency as u8);
                    self.square_wave.set_freq_upper((frequency >> 8) as u8 & 0x7);
                }
            }
        }
        self.square_wave.tick();
        self.volume_envelope.tick();
    }

    pub fn sample(&self) -> u8 {
        if self.enabled {
            self.volume_envelope.apply(self.square_wave.sample())
        } else {
            0
        }
    }

    pub fn update_sweep(&mut self, byte: u8) {
        self.sweep.update(byte);
    }

    pub fn set_length(&mut self, length: u8) {
        self.length_counter.set_length(length);
    }

    pub fn set_length_enabled(&mut self, enabled: bool, fs: &FrameSequencer) {
        match self.length_counter.set_enabled(enabled, fs.next_is_length_counter()) {
            LengthCounterAction::Nop => {}
            LengthCounterAction::Disable => {
                self.enabled = false;
            }
        }
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
        self.dac_on = self.volume_envelope.is_on();
        if !self.dac_on {
            self.enabled = false;
        }
    }

    pub fn trigger(&mut self, will_enable: bool) {
        if self.dac_on {
            self.enabled = true;
        }
        self.length_counter.trigger(will_enable);
        match self.sweep.trigger(self.square_wave.frequency) {
            SweepAction::Disable => {
                self.enabled = false
            }
            _ => { }
        }
    }
}
