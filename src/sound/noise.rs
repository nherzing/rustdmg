use crate::clocks::CLOCK_FREQ;
use super::envelope::VolumeEnvelope;

const DIVISORS: [u32; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

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
            bits: 0xFFFF,
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
            let oldbits = self.bits;
            self.bits = (bit << (self.len-1)) | (self.bits >> 1);
        } else {
            self.timer -= 1;
        }
    }

    fn sample(&self) -> u8 {
        (self.bits & 0x1) as u8
    }

    fn reset(&mut self) {
        self.bits = 0xFFFF;
        self.timer = self.frequency();
    }

    fn frequency(&self) -> u16 {
        ((CLOCK_FREQ / DIVISORS[self.divisor_code]) >> self.freq_shift) as u16
    }
}

pub struct Noise {
    playing: bool,
    lfsr: Lfsr,
    volume_envelope: VolumeEnvelope,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            playing: false,
            lfsr: Lfsr::new_from_byte(0x00),
            volume_envelope: VolumeEnvelope::new_from_byte(0x00)
        }
    }

    pub fn tick(&mut self) {
        if !self.playing {
            return
        }

        self.lfsr.tick();
        self.volume_envelope.tick();
    }

    pub fn sample(&self) -> u8 {
        if self.playing {
            self.volume_envelope.apply(self.lfsr.sample())
        } else {
            0
        }
    }

    pub fn set_lsrf(&mut self, lfsr: Lfsr) {
        self.lfsr = lfsr;
    }

    pub fn set_volume_envelope(&mut self, volume_envelope: VolumeEnvelope) {
        self.volume_envelope = volume_envelope;
    }

    pub fn restart(&mut self) {
        self.playing = true;
        self.lfsr.reset();
    }
}
