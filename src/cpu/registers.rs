#[derive(Copy, Clone, Debug)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC
}

#[derive(Default)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16
}

use Register::*;

impl Registers {
    pub fn new() -> Registers {
        Default::default()
    }

    pub fn get8(&self, r: Register) -> u8 {
        match r {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            F => self.f,
            H => self.h,
            L => self.l,
            _ => panic!("Can't read 8 bit value from 16 bit register {:?}", r)
        }
    }

    pub fn get16(&self, r: Register) -> u16 {
        match r {
            AF => self.combine(self.a, self.f),
            BC => self.combine(self.b, self.c),
            DE => self.combine(self.d, self.e),
            HL => self.combine(self.h, self.l),
            SP => self.sp,
            PC => self.pc,
            _ => panic!("Can't read 16 bit value from 8 bit register {:?}", r)
        }
    }

    pub fn set16(&mut self, r: Register, v: u16) {
        match r {
            AF => self.split(A, F, v),
            BC => self.split(B, C, v),
            DE => self.split(D, E, v),
            HL => self.split(H, L, v),
            SP => self.sp = v,
            PC => self.pc = v,
            _ => panic!("Can't set 16 bit value into 8 bit register {:?}", r)
        }
    }

    pub fn set8(&mut self, r: Register, v: u8) {
        match r {
            A => self.a = v,
            B => self.b = v,
            C => self.c = v,
            D => self.d = v,
            E => self.e = v,
            F => self.f = v,
            H => self.h = v,
            L => self.l = v,
            _ => panic!("Can't set 16 bit register {:?} with 8 bit value", r)
        }
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn inc_pc(&mut self, n: u16) {
        self.pc += n;
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, cy: bool) {
        self.f = ((z as u8) << 7) | ((n as u8) << 6) | ((h as u8) << 5) | ((cy as u8) << 4);
    }

    pub fn z_flag(&self) -> u8 {
        (self.f >> 7) & 1
    }

    pub fn n_flag(&self) -> u8 {
        (self.f >> 6) & 1
    }

    pub fn h_flag(&self) -> u8 {
        (self.f >> 5) & 1
    }

    pub fn cy_flag(&self) -> u8 {
        (self.f >> 4) & 1
    }

    fn combine(&self, a: u8, b: u8) -> u16 {
        ((a as u16) << 8) | (b as u16)
    }

    fn split(&mut self, rh: Register, rl: Register, v: u16) {
        self.set8(rh, (v >> 8) as u8);
        self.set8(rl, (v & 0xFF) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get8_set8() {
        let mut rs = Registers::new();
        rs.set8(A, 42);
        rs.set8(H, 43);
        assert_eq!(rs.get8(A), 42);
        assert_eq!(rs.get8(H), 43);
    }

    #[test]
    #[should_panic]
    fn test_bad_set8() {
        let mut rs = Registers::new();
        rs.set8(AF, 42);
    }

    #[test]
    #[should_panic]
    fn test_bad_get8() {
        let rs = Registers::new();
        rs.get8(AF);
    }

    #[test]
    fn test_get16_set16() {
        let mut rs = Registers::new();
        rs.set16(AF, 0xDEAD);
        rs.set16(SP, 0xBEEF);
        assert_eq!(rs.get16(AF), 0xDEAD);
        assert_eq!(rs.get8(A), 0xDE);
        assert_eq!(rs.get8(F), 0xAD);
        assert_eq!(rs.get16(SP), 0xBEEF);
    }

    #[test]
    #[should_panic]
    fn test_bad_set16() {
        let mut rs = Registers::new();
        rs.set16(A, 42);
    }

    #[test]
    #[should_panic]
    fn test_bad_get16() {
        let rs = Registers::new();
        rs.get16(F);
    }

}
