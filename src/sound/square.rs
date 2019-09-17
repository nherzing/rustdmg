use crate::clocks::CLOCK_FREQ;
use super::envelope::VolumeEnvelope;

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
    playing: bool,
    square_wave: WaveGen,
    volume_envelope: VolumeEnvelope
}

impl SquareWave {
    pub fn new() -> Self {
        SquareWave {
            playing: false,
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
    }

    pub fn sample(&self) -> u8 {
        if self.playing {
            self.volume_envelope.apply(self.square_wave.sample())
        } else {
            0
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
    }

    pub fn restart(&mut self) {
        self.playing = true;
    }

}
