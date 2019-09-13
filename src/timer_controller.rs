use crate::memory::memory_map::{MemoryMappedDevice, MappedArea};
use crate::interrupt_controller::Interrupt;
use crate::clocks::CLOCK_FREQ;

const DIV_FREQ: u32 = 16384;
const TAC_FREQS: [u32; 4] = [4096, 262144, 65536, 16384];
const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;

struct Timer {
    clocks_per_tick: u32,
    clocks_left: u32,
    pub value: u8,
    pub default_value: u8
}

impl Timer {
    fn new(freq: u32, default_value: u8) -> Timer {
        let clocks_per_tick = CLOCK_FREQ / freq;
        Timer {
            clocks_per_tick,
            clocks_left: clocks_per_tick,
            value: 0,
            default_value
        }
    }

    fn tick(&mut self, clocks: u32) -> bool {
        let mut to_tick = clocks;
        let mut overflow = false;
        while to_tick > 0 {
            if self.clocks_left <= to_tick {
                to_tick = to_tick - self.clocks_left;
                overflow |= self.value == 0xFF;
                self.value = if self.value == 0xFF { self.default_value } else { self.value + 1 };
                self.clocks_left = self.clocks_per_tick;
            } else {
                self.clocks_left -= to_tick;
                to_tick = 0;
            }
        }
        overflow
    }

    fn set(&mut self, value: u8) {
        self.value = value;
    }
}

pub struct TimerController {
    tac: u8,
    div_ticker: Timer,
    tima_ticker: Timer,
    tima_running: bool
}

impl TimerController {
    pub fn new() -> TimerController {
        TimerController {
            tac: 0,
            div_ticker: Timer::new(DIV_FREQ, 0),
            tima_ticker: Timer::new(TAC_FREQS[0], 0),
            tima_running: false
        }
    }

    pub fn mapped_areas() -> [MappedArea; 1] {
        [
            MappedArea(DIV, 4)
        ]
    }

    pub fn tick<F>(&mut self, clocks: u32, mut fire_interrupt: F) where
    F: FnMut(Interrupt) {
        self.div_ticker.tick(clocks);
        if self.tima_running {
            if self.tima_ticker.tick(clocks) {
                fire_interrupt(Interrupt::Timer)
            }
        };
    }
}


impl MemoryMappedDevice for TimerController {
    fn get8(&self, addr: u16) -> u8 {
        match addr {
            DIV => self.div_ticker.value,
            TIMA => self.tima_ticker.value,
            TMA => self.tima_ticker.default_value,
            TAC => self.tac,
            _ => { panic!("Invalid get address 0x{:X} mapped to TimerController", addr) }
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            DIV => { self.div_ticker.set(0); }
            TIMA => { self.tima_ticker.set(byte); }
            TMA => { self.tima_ticker.default_value = byte; }
            TAC => {
                self.tac = byte;
                self.tima_running = b2!(byte) == 1;
                self.tima_ticker = Timer::new(
                    TAC_FREQS[(self.tac & 0b11) as usize],
                    self.tima_ticker.default_value
                );
            }
            _ => { panic!("Invalid set address 0x{:X} mapped to TimerController", addr) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_div() {
        let mut t = TimerController::new();

        assert_eq!(t.get8(DIV), 0);
        t.tick(32);
        assert_eq!(t.get8(DIV), 0);
        t.tick(31);
        assert_eq!(t.get8(DIV), 0);
        t.tick(1);
        assert_eq!(t.get8(DIV), 1);
    }

    #[test]
    fn test_div_big_tick() {
        let mut t = TimerController::new();

        t.tick(64*5 + 63);
        assert_eq!(t.get8(DIV), 5);
        t.tick(1);
        assert_eq!(t.get8(DIV), 6);
    }

    #[test]
    fn test_div_reset() {
        let mut t = TimerController::new();

        t.tick(64*5 + 4);
        assert_eq!(t.get8(DIV), 5);
        t.set8(DIV, 42);
        assert_eq!(t.get8(DIV), 0);
        t.tick(61);
        assert_eq!(t.get8(DIV), 1);
    }

    #[test]
    fn test_div_overflow() {
        let mut t = TimerController::new();

        t.tick(64*255 + 63);
        assert_eq!(t.get8(DIV), 255);
        t.tick(2);
        assert_eq!(t.get8(DIV), 0);
        t.tick(63);
        assert_eq!(t.get8(DIV), 1);
    }
}
