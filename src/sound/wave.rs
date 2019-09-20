use super::frame_sequencer::FrameSequencer;
use super::length_counter::{LengthCounter, LengthCounterAction};

#[derive(Clone, Copy, Debug)]
enum Volume {
    Mute,
    Quarter,
    Half,
    Full
}

use Volume::*;

const VOLUMES: [Volume; 4] = [Mute, Full, Half, Quarter];

#[derive(Debug)]
struct CustomWave {
    wave_table: [u8; 32],
    frequency: u16,
    timer: u16,
    cycle_offset: usize
}

impl CustomWave {
    fn new() -> Self {
        CustomWave {
            wave_table: [0; 32],
            frequency: 0,
            timer: 0,
            cycle_offset: 0
        }
    }

    fn set_freq_lower(&mut self, data: u8) {
        self.frequency = (self.frequency & 0x700) | (data as u16);
    }

    fn set_freq_upper(&mut self, data: u8) {
        self.frequency = (self.frequency & 0xFF) | ((data as u16) << 8);
    }

    fn set_wave(&mut self, offset: u8, data: u8) {
        let idx = (offset as usize) * 2;
        self.wave_table[idx] = (data >> 4) & 0xF;
        self.wave_table[idx + 1] = data & 0xF;
    }

    fn tick(&mut self) {
        if self.timer == 0 {
            self.cycle_offset = (self.cycle_offset + 1) % 32;
            self.timer = (2048 - self.frequency)/2;
        } else {
            self.timer -= 1;
        }
    }

    fn sample(&self) -> u8 {
        self.wave_table[self.cycle_offset]
    }

    fn reset(&mut self) {
        self.cycle_offset = 0;
        self.timer = (2048 - self.frequency)*2;
    }
}

#[derive(Debug)]
pub struct Wave {
    dac_on: bool,
    enabled: bool,
    custom_wave: CustomWave,
    volume: Volume,
    length_counter: LengthCounter,
    frequency: u16
}

impl Wave {
    pub fn new() -> Self {
        Wave {
            dac_on: false,
            enabled:false,
            custom_wave: CustomWave::new(),
            volume: Mute,
            length_counter: LengthCounter::new(256),
            frequency: 0
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
                }
            }
        }

        if !self.dac_on || !self.enabled {
            return
        }

        self.custom_wave.tick();
    }

    pub fn sample(&self) -> u8 {
        if self.enabled {
            let sample = self.custom_wave.sample();
            match self.volume {
                Mute => 0,
                Quarter => sample >> 2,
                Half => sample >> 1,
                Full => sample
            }
        } else {
            0
        }
    }

    pub fn set_dac(&mut self, dac_on: bool) {
        self.dac_on = dac_on;
        if !self.dac_on {
            self.enabled = false;
        }
    }

    pub fn set_volume(&mut self, volume_code: u8) {
        self.volume = VOLUMES[volume_code as usize];
    }

    pub fn set_wave(&mut self, offset: u8, data: u8) {
        self.custom_wave.set_wave(offset, data);
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

    pub fn set_freq_lower(&mut self, data: u8) {
        self.custom_wave.set_freq_lower(data);
    }

    pub fn set_freq_upper(&mut self, data: u8) {
        self.custom_wave.set_freq_upper(data);
    }

    pub fn trigger(&mut self) {
        if self.dac_on {
            self.enabled = true;
        }
        self.length_counter.trigger();
        self.custom_wave.reset();
    }
}
