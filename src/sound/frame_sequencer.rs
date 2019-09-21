use crate::clocks::CLOCK_FREQ;

pub struct FrameSequencer {
    ticks_left: u32,
    step: u32,
    clock: Option<u32>
}

impl FrameSequencer {
    pub fn new() -> Self {
        FrameSequencer {
            ticks_left: 0,
            step: 0,
            clock: None
        }
    }

    pub fn tick(&mut self) {
        if self.ticks_left == 0 {
            self.ticks_left = CLOCK_FREQ / 512;
            self.clock = Some(self.step);
            debug!("CLOCK: {}", self.step);
            self.step = (self.step + 1) % 8;
        } else {
            self.clock = None;
        }
        self.ticks_left -= 1;
    }

    pub fn is_length_counter(&self) -> bool {
        self.clock.map_or(false, |v| v % 2 == 0)
    }

    pub fn is_sweep(&self) -> bool {
        self.clock.map_or(false, |v| v == 2 || v == 6)
    }

    pub fn next_is_length_counter(&self) -> bool {
        self.step % 2 == 0
    }
}
