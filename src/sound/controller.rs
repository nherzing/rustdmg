use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use super::frame_sequencer::{FrameSequencer};
use super::square::{SquareWave};
use super::wave::{Wave};
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

const REG_ORS: [u8; 48] = [
    0x80, 0x3F, 0x00, 0xFF, 0xBF,
    0xFF, 0x3F, 0x00, 0xFF, 0xBF,
    0x7F, 0xFF, 0x9F, 0xFF, 0xBF,
    0xFF, 0xFF, 0x00, 0x00, 0xBF,
    0x00, 0x00, 0x70, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00
];

pub struct SoundController {
    regs: [u8; 48],
    on: bool,
    square_a: SquareWave,
    square_b: SquareWave,
    wave: Wave,
    noise: Noise,
    nr50: u8,
    nr51: u8,
    frame_sequencer: FrameSequencer
}

impl SoundController {
    pub fn new() -> Self {
        SoundController {
            regs: [0; 48],
            on: false,
            square_a: SquareWave::new(),
            square_b: SquareWave::new(),
            wave: Wave::new(),
            noise: Noise::new(),
            nr50: 0,
            nr51: 0,
            frame_sequencer: FrameSequencer::new()
        }
    }

    pub fn mapped_areas() -> [MappedArea; 1] {
        [
            MappedArea(NR10, 48)
        ]
    }

    pub fn tick(&mut self, clocks: u32, audio_queue: &mut Vec<f32>) {
        if !self.on {
            audio_queue.push(0.0);
            audio_queue.push(0.0);
            return
        }

        for _ in 0..clocks {
            self.frame_sequencer.tick();
            self.square_a.tick(&self.frame_sequencer);
            self.square_b.tick(&self.frame_sequencer);
            self.wave.tick(&self.frame_sequencer);
            self.noise.tick(&self.frame_sequencer);
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

            audio_queue.push(l_sample / 16.0);
            audio_queue.push(r_sample / 16.0);
        }
    }
}

impl MemoryMappedDevice for SoundController {
    fn set8(&mut self, addr: u16, byte: u8) {
        if addr == NR52 {
            if b7!(byte) == 0 {
                for i in NR10..NR52 {
                    self.set8(i, 0);
                }
            }
            self.on = b7!(byte) == 1;
            return
        }
        if !self.on { return }
        self.regs[(addr - NR10) as usize] = byte;
        match addr {
            NR50 => {
                self.nr50 = byte;
            }
            NR51 => {
                self.nr51 = byte;
            }
            NR10 => {
                self.square_a.update_sweep(byte);
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
                if b7!(byte) == 1 {
                    self.square_a.trigger(b6!(byte) == 1);
                }
                self.square_a.set_length_enabled(b6!(byte) == 1, &self.frame_sequencer);
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
                if b7!(byte) == 1 {
                    self.square_b.trigger(false);
                }

                self.square_b.set_freq_upper(byte & 0x7);
                self.square_b.set_length_enabled(b6!(byte) == 1, &self.frame_sequencer);            }
            NR30 => {
                self.wave.set_dac(b7!(byte) == 1);
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
                if b7!(byte) == 1 {
                    self.wave.trigger(false);
                }

                self.wave.set_freq_upper(byte & 0x7);
                self.wave.set_length_enabled(b6!(byte) == 1, &self.frame_sequencer);
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
                if b7!(byte) == 1 {
                    self.noise.trigger(false);
                }

                self.noise.set_length_enabled(b6!(byte) == 1, &self.frame_sequencer);
            }
            _ => {
                debug!("Invalid set address 0x:{:X} mapped to Sound Controller", addr);
            }
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        let offset = (addr - NR10) as usize;
        match addr {
            NR52 => {
                let mut val = (if self.on { 1 << 7 } else { 0 }) | REG_ORS[offset];
                if self.square_a.is_on() {
                    val |= 0x1;
                }
                if self.square_b.is_on() {
                    val |= 0x2;
                }
                if self.wave.is_on() {
                    val |= 0x4;
                }
                if self.noise.is_on() {
                    val |= 0x8;
                }
                val
            }
            NR10 ... WAVE_END => {
                let val = self.regs[offset] | REG_ORS[offset];
                val
            }
            _ => panic!("Invalid get address 0x{:X} mapped to Sound Controller", addr)
        }
    }
}
