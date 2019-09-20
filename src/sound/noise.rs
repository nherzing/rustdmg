use super::frame_sequencer::FrameSequencer;
use crate::clocks::CLOCK_FREQ;
use super::envelope::VolumeEnvelope;
use super::length_counter::{LengthCounter, LengthCounterAction};

#[derive(Debug)]
pub struct Lfsr {
    bits: u16,
    freq_shift: u8,
    len: u8,
    divisor_code: usize,
    timer: u16,
}

impl Lfsr {
    pub fn new_from_byte(byte: u8) -> Self {
        Lfsr {
            bits: 0x0000,
            freq_shift: (byte >> 4) & 0xF,
            len: if b3!(byte) == 0 { 15 } else { 7 },
            divisor_code: (byte & 0x7) as usize,
            timer: 0
        }
    }

    fn tick(&mut self) {
        if self.timer == 0 {
            self.timer = self.frequency();
            let bit = (self.bits & 0x1) ^ ((self.bits >> 1) & 0x1);
            self.bits = if self.len == 7 {
                (bit << (7-1)) | ((self.bits >> 1) & 0x3FBF) // 0b011111110111111
            } else {
                (bit << (15-1)) | ((self.bits >> 1) & 0x3FFF)
            };
        } else {
            self.timer -= 1;
        }
    }

    fn sample(&self) -> u8 {
        ((self.bits & 0x1) ^ 0x1) as u8
    }

    fn reset(&mut self) {
        self.bits = 0x7FFF;
        self.timer = self.frequency();
    }

    fn frequency(&self) -> u16 {
        let c = 524288;
        let d = if self.divisor_code == 0 { c * 2 } else { c / (self.divisor_code as u32) };
        let hz = d >> (self.freq_shift + 1);
        (CLOCK_FREQ / hz) as u16
    }
}

pub struct Noise {
    dac_on: bool,
    enabled: bool,
    lfsr: Lfsr,
    volume_envelope: VolumeEnvelope,
    length_counter: LengthCounter
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            dac_on: false,
            enabled: false,
            lfsr: Lfsr::new_from_byte(0xFF),
            volume_envelope: VolumeEnvelope::new_from_byte(0x00),
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

        self.lfsr.tick();
        self.volume_envelope.tick();
    }

    pub fn sample(&self) -> u8 {
        if self.enabled {
            self.volume_envelope.apply(self.lfsr.sample())
        } else {
            0
        }
    }

    pub fn set_lsrf(&mut self, lfsr: Lfsr) {
        self.lfsr = lfsr;
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

    pub fn set_volume_envelope(&mut self, volume_envelope: VolumeEnvelope) {
        self.volume_envelope = volume_envelope;
        self.dac_on = self.volume_envelope.is_on();
        if !self.dac_on {
            self.enabled = false;
        }
    }

    pub fn restart(&mut self) {
        if self.dac_on {
            self.enabled = true;
        }
        self.length_counter.trigger();
        self.lfsr.reset();
    }
}
