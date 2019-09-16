use crate::memory::memory_map::{MemoryMappedDevice};
use crate::memory::memory_map::{MappedArea};
use crate::clocks::CLOCK_FREQ;

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
const SILENT: u8 = 0;
const HIGH: u8 = 1;
const LOW: u8 = 0;

const DUTY_CYCLES: [[u8; 8]; 4] = [
    [LOW, LOW, LOW, LOW, LOW, LOW, LOW, HIGH],
    [HIGH, LOW, LOW, LOW, LOW, LOW, LOW, LOW],
    [HIGH, LOW, LOW, LOW, LOW, HIGH, HIGH, HIGH],
    [LOW, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, LOW]
];

#[derive(Debug)]
enum Direction {
    Increase,
    Decrease
}

use Direction::*;

#[derive(Debug)]
struct VolumeEnvelope {
    volume: u8,
    direction: Direction,
    period: u8,
    ticks_left: u32
}

impl VolumeEnvelope {
    fn new_from_byte(byte: u8) -> Self {
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

    fn tick(&mut self) {
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

    fn apply(&self, input: u8) -> u8 {
        input * self.volume
    }
 }

struct SquareWave {
    frequency: u16,
    timer: u16,
    duty_index: usize,
    cycle_offset: usize
}

impl SquareWave {
    fn new() -> Self {
        SquareWave {
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

struct PulseWave {
    playing: bool,
    square_wave: SquareWave,
    volume_envelope: VolumeEnvelope
}

impl PulseWave {
    fn new() -> Self {
        PulseWave {
            playing: false,
            square_wave: SquareWave::new(),
            volume_envelope: VolumeEnvelope::new_from_byte(0xF3)
        }
    }

    fn tick(&mut self) {
        if !self.playing {
            return
        }

        self.square_wave.tick();
        self.volume_envelope.tick();
    }

    fn sample(&self) -> u8 {
        if self.playing {
            self.volume_envelope.apply(self.square_wave.sample())
        } else {
            SILENT
        }
    }

    fn set_duty(&mut self, duty_index: u8) {
        self.square_wave.set_duty(duty_index);
    }

    fn set_freq_lower(&mut self, data: u8) {
        self.square_wave.set_freq_lower(data);
    }

    fn set_freq_upper(&mut self, data: u8) {
        self.square_wave.set_freq_upper(data);
    }

    fn set_volume_envelope(&mut self, volume_envelope: VolumeEnvelope) {
        debug!("Set: {:?}", volume_envelope);
        self.volume_envelope = volume_envelope;
    }

    fn restart(&mut self) {
        self.playing = true;
    }

}

pub struct SoundController {
    on: bool,
    pulse_a: PulseWave
}

impl SoundController {
    pub fn new() -> Self {
        SoundController {
            on: false,
            pulse_a: PulseWave::new()
        }
    }

    pub fn mapped_areas() -> [MappedArea; 5] {
        [
            MappedArea(NR10, 5),
            MappedArea(NR21, 4),
            MappedArea(NR30, 5),
            MappedArea(NR41, 7),
            MappedArea(0xFF30, 16)
        ]
    }

    pub fn tick(&mut self, clocks: u32) -> Vec::<f32> {
        if !self.on { return vec![0.0, 0.0] }
        let mut result = Vec::new();

        for _ in 0..clocks {
            self.pulse_a.tick();
            let sample = self.pulse_a.sample();
            let v = (sample as f32) / 15.0;
            result.push(v);
            result.push(v);
        }

        result
    }
}

impl MemoryMappedDevice for SoundController {
    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            NR50 => {
                let so2_volume = (byte >> 4) & 0x7;
                let so1_volume = byte & 0x7;
                debug!("Set NR50: {:08b}, so2: {}, so1: {}", byte, so2_volume, so1_volume);
            }
            NR51 => {
                debug!("Set NR51: {:08b} (output selection)", byte);
            }
            NR52 => {
                self.on = b7!(byte) == 1;
                debug!("Set NR52: on: {}", self.on);
            }
            NR11 => {
                let duty = byte >> 6;
                let length = byte & 0x3F;
                debug!("Set NR11: {:08b}, duty: {}, length: {}", byte, duty, length);
                self.pulse_a.set_duty(byte >> 6);
                // SET LENGTH
            }
            NR12 => {
                let volume = byte >> 4;
                let direction = if b3!(byte) == 0 { Decrease } else { Increase };
                let period = byte & 0x7;
                debug!("Set NR12: {:08b}, volume: {}, direction: {:?}, period: {}", byte, volume, direction, period);
                self.pulse_a.set_volume_envelope(VolumeEnvelope::new_from_byte(byte));
            }
            NR13 => {
                debug!("Set NR13: {:08b}", byte);
                self.pulse_a.set_freq_lower(byte);
            }
            NR14 => {
                debug!("Set NR14: {:08b}", byte);
                self.pulse_a.set_freq_upper(byte & 0x7);
                if b7!(byte) == 1 {
                    self.pulse_a.restart();
                }
            }
            _ => panic!("Invalid set address 0x{:X}: 0x{:X} mapped to Sound Controller", addr, byte)
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            _ => panic!("Invalid get address 0x{:X} mapped to Sound Controller", addr)
        }
    }
}
