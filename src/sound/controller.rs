use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use super::square::{SquareWave};
use super::wave::{Wave};
use super::sweep::{Sweep};
use super::noise::{Noise, Lfsr};
use super::envelope::VolumeEnvelope;

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;
const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;
const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;
const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;
const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;
const WAVE_START: u16 = 0xFF30;
const WAVE_END: u16 = 0xFF3F;
const SILENT: u8 = 0;
const HIGH: u8 = 1;
const LOW: u8 = 0;

pub struct SoundController {
    on: bool,
    square_a: SquareWave,
    square_b: SquareWave,
    wave: Wave,
    noise: Noise,
    nr50: u8,
    nr51: u8
}

impl SoundController {
    pub fn new() -> Self {
        SoundController {
            on: false,
            square_a: SquareWave::new(),
            square_b: SquareWave::new(),
            wave: Wave::new(),
            noise: Noise::new(),
            nr50: 0,
            nr51: 0
        }
    }

    pub fn mapped_areas() -> [MappedArea; 5] {
        [
            MappedArea(NR10, 5),
            MappedArea(NR21, 4),
            MappedArea(NR30, 5),
            MappedArea(NR41, 7),
            MappedArea(WAVE_START, 16)
        ]
    }

    pub fn tick(&mut self, clocks: u32) -> Vec::<f32> {
        if !self.on { return vec![0.0, 0.0] }
        let mut result = Vec::new();

        for _ in 0..clocks {
            self.square_a.tick();
            self.square_b.tick();
            self.wave.tick();
            self.noise.tick();
            let mut l_sample = 0.0;
            let mut r_sample = 0.0;

            let scale = |s| ((s as f32) / 15.0) * 0.75;
            let square_a_sample = scale(self.square_a.sample());
            let square_b_sample = scale(self.square_b.sample());
            let wave_sample = scale(self.wave.sample());
            let noise_sample = scale(self.noise.sample());

            if b4!(self.nr51) == 1 {
                l_sample += square_a_sample;
            }
            if b0!(self.nr51) == 1 {
                r_sample += square_a_sample;
            }
            if b5!(self.nr51) == 1 {
                l_sample += square_b_sample;
            }
            if b1!(self.nr51) == 1 {
                r_sample += square_b_sample;
            }
            if b6!(self.nr51) == 1 {
                l_sample += wave_sample;
            }
            if b2!(self.nr51) == 1 {
                r_sample += wave_sample;
            }
            if b7!(self.nr51) == 1 {
                l_sample += noise_sample;
            }
            if b3!(self.nr51) == 1 {
                r_sample += noise_sample;
            }

            let l_volume = ((self.nr50 >> 4) & 0x7) + 1;
            let r_volume = (self.nr50 & 0x7) + 1;

            l_sample *= l_volume as f32;
            r_sample *= r_volume as f32;

            result.push(l_sample / 16.0);
            result.push(r_sample / 16.0);
        }

        result
    }
}

impl MemoryMappedDevice for SoundController {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            NR50 => {
                self.nr50 = byte;
            }
            NR51 => {
                self.nr51 = byte;
            }
            NR52 => {
                self.on = b7!(byte) == 1;
            }
            NR10 => {
                self.square_a.set_sweep(Sweep::new_from_byte(byte));
            }
            NR11 => {
                self.square_a.set_duty(byte >> 6);
                self.square_a.set_length(byte & 0x3F);
            }
            NR12 => {
                self.square_a.set_volume_envelope(VolumeEnvelope::new_from_byte(byte));
            }
            NR13 => {
                self.square_a.set_freq_lower(byte);
            }
            NR14 => {
                self.square_a.set_freq_upper(byte & 0x7);
                self.square_a.set_length_enabled(b6!(byte) == 1);
                if b7!(byte) == 1 {
                    self.square_a.restart();
                }
            }
            NR21 => {
                self.square_b.set_duty(byte >> 6);
                self.square_b.set_length(byte & 0x3F);
            }
            NR22 => {
                self.square_b.set_volume_envelope(VolumeEnvelope::new_from_byte(byte));
            }
            NR23 => {
                self.square_b.set_freq_lower(byte);
            }
            NR24 => {
                self.square_b.set_freq_upper(byte & 0x7);
                self.square_b.set_length_enabled(b6!(byte) == 1);
                if b7!(byte) == 1 {
                    self.square_b.restart();
                }
            }
            NR30 => {
                self.wave.set_playing(b7!(byte) == 1);
            }
            NR31 => {
                self.wave.set_length(byte);
            }
            NR32 => {
                self.wave.set_volume((byte >> 5) & 0x3);
            }
            NR33 => {
                self.wave.set_freq_lower(byte);
            }
            NR34 => {
                self.wave.set_freq_upper(byte & 0x7);
                self.wave.set_length_enabled(b6!(byte) == 1);
                if b7!(byte) == 1 {
                    self.wave.trigger();
                }
            }
            WAVE_START ... WAVE_END => {
                self.wave.set_wave((addr - WAVE_START) as u8, byte);
            }
            NR41 => {
                self.noise.set_length(byte & 0x3F);
            }
            NR42 => {
                self.noise.set_volume_envelope(VolumeEnvelope::new_from_byte(byte));
            }
            NR43 => {
                self.noise.set_lsrf(Lfsr::new_from_byte(byte));
            }
            NR44 => {
                self.noise.set_length_enabled(b6!(byte) == 1);
                if b7!(byte) == 1 {
                    self.noise.restart();
                }
            }
            _ => {}
            _ => panic!("Invalid set address 0x{:X}: 0x{:X} mapped to Sound Controller", addr, byte)
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            _ => panic!("Invalid get address 0x{:X} mapped to Sound Controller", addr)
        }
    }
}
