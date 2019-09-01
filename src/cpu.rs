use crate::memory_bus::{MemoryBus};
use crate::registers::{Register, Registers};
use crate::opcode::{Src, FlagCondition, Opcode, Instr};

use Opcode::*;
use Register::*;

struct Cpu {
    memory_bus: MemoryBus,
    registers: Registers,
    ime: bool
}

impl Cpu {
    pub fn new(memory_bus: MemoryBus) -> Cpu {
        Cpu {
            memory_bus,
            registers: Registers::new(),
            ime: false
        }
    }

    pub fn step(&mut self) -> u8 {
        let instr = self.disassemble();
        let mut inc_pc = true;
        let mut cycles = instr.cycles;

        match instr.opcode {
            NOP => {}
            HALT => {}
            LD8(dst, src) => {
                self.ld8(dst, src);
            }
            LD8I(dst, src) => {
                self.ld8(dst, src);
                self.registers.set16(HL, self.registers.get16(HL) + 1);
            }
            LD8D(dst, src) => {
                self.ld8(dst, src);
                self.registers.set16(HL, self.registers.get16(HL) - 1);
            },
            LD16(dst, src) => {
                self.ld16(dst, src);
            }
            PUSH(reg) => {
                let sp = self.registers.get16(SP);
                self.registers.set16(SP, sp - 2);
                self.memory_bus.set16(sp - 2, self.registers.get16(reg));
            },
            POP(reg) => {
                let sp = self.registers.get16(SP);
                self.registers.set16(SP, sp + 2);
                self.registers.set16(reg, self.memory_bus.get16(sp));
            }
            ADD8(src) => {
                let a = self.registers.get8(A) as u16;
                let b = self.get_u8(src) as u16;
                let r16 = a + b;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) + (b & 0xF);
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, false, (r4 >> 4) == 1, (r16 >> 8) == 1);
            }
            ADC8(src) => {
                let a = self.registers.get8(A) as u16;
                let b = self.get_u8(src) as u16;
                let cy = self.registers.cy_flag() as u16;
                let r16 = a + b + cy;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) + (b & 0xF) + cy;
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, false, (r4 >> 4) == 1, (r16 >> 8) == 1);
            }
            SUB8(src) => {
                let a = self.registers.get8(A) as i16;
                let b = self.get_u8(src) as i16;
                let r16 = a - b;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) - (b & 0xF);
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, true, r4 < 0, r16 < 0);
            }
            SBC8(src) => {
                let a = self.registers.get8(A) as i16;
                let b = self.get_u8(src) as i16;
                let cy = self.registers.cy_flag() as i16;
                let r16 = a - b - cy;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) - (b & 0xF) - cy;
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, true, r4 < 0, r16 < 0);
            }
            AND(src) => {
                let a = self.registers.get8(A);
                let b = self.get_u8(src);
                let r8 = a & b;
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, false, true, false);
            }
            OR(src) => {
                let a = self.registers.get8(A);
                let b = self.get_u8(src);
                let r8 = a | b;
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, false, false, false);
            }
            XOR(src) => {
                let a = self.registers.get8(A);
                let b = self.get_u8(src);
                let r8 = a ^ b;
                self.registers.set8(A, r8);
                self.registers.set_flags(r8 == 0, false, false, false);
            }
            CP(src) => {
                let a = self.registers.get8(A) as i16;
                let b = self.get_u8(src) as i16;
                let r16 = a - b;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) - (b & 0xF);
                self.registers.set_flags(r8 == 0, true, r4 < 0, r16 < 0);
            }
            INC8(dst) => {
                let a = self.get_u8(dst) as u16;
                let r16 = a + 1;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) + 1;
                self.ld8(dst, Src::D8(r8));
                self.registers.set_flags(r8 == 0, false, (r4 >> 4) == 1, self.registers.cy_flag() == 1);
            }
            DEC8(dst) => {
                let a = self.get_u8(dst) as i16;
                let r16 = a - 1;
                let r8 = (r16 & 0xFF) as u8;
                let r4 = (a & 0xF) - 1;

                self.ld8(dst, Src::D8(r8));
                self.registers.set_flags(r8 == 0, true, r4 < 0, self.registers.cy_flag() == 1);
            }
            ADD16(reg, Src::I8(v)) => {
                let a = self.registers.get16(reg) as i32;
                let b = v as i32;
                let r32 = a + b;
                let r16 = (r32 & 0xFFFF) as u16;
                let r12 = (a & 0xFFF) + (b & 0xFFF);

                self.registers.set16(reg, r16);
                // not sure about this carry
                self.registers.set_flags(false, false, (r12 >> 12) == 1, (r32 >> 16) == 1);
            }
            ADD16(reg, src) => {
                let a = self.registers.get16(reg) as u32;
                let b = self.get_u16(src) as u32;
                let r32 = a + b;
                let r16 = (r32 & 0xFFFF) as u16;
                let r12 = (a & 0xFFF) + (b & 0xFFF);

                self.registers.set16(reg, r16);
                self.registers.set_flags(self.registers.z_flag() == 1, false, (r12 >> 12) == 1, (r32 >> 16) == 1);
            }
            INC16(reg) => {
                self.registers.set16(reg, self.registers.get16(reg) + 1);
            }
            DEC16(reg) => {
                self.registers.set16(reg, self.registers.get16(reg) - 1);
            }
            RLCA => {
                let a = self.registers.get8(A);
                let b7 = (a >> 7) & 1;
                self.registers.set8(A, (a << 1) | b7);
                self.registers.set_flags(false, false, false, b7 == 1);
            }
            RLA => {
                let a = self.registers.get8(A);
                let b7 = (a >> 7) & 1;
                self.registers.set8(A, (a << 1) | self.registers.cy_flag());
                self.registers.set_flags(false, false, false, b7 == 1);
            }
            RRCA => {
                let a = self.registers.get8(A);
                let b0 = a & 1;
                self.registers.set8(A, (a >> 1) | (b0 << 7));
                self.registers.set_flags(false, false, false, b0 == 1);
            }
            RRA => {
                let a = self.registers.get8(A);
                let b0 = a & 1;
                self.registers.set8(A, (a >> 1) | (self.registers.cy_flag() << 7));
                self.registers.set_flags(false, false, false, b0 == 1);
            }
            RLC(dst) => {
                let a = self.get_u8(dst);
                let b7 = (a >> 7) & 1;
                let r = (a << 1) | b7;
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b7 == 1);
            }
            RL(dst) => {
                let a = self.get_u8(dst);
                let b7 = (a >> 7) & 1;
                let r = (a << 1) | self.registers.cy_flag();
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b7 == 1);
            }
            RRC(dst) => {
                let a = self.get_u8(dst);
                let b0 = a & 1;
                let r = (a >> 1) | (b0 << 7);
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b0 == 1);
            }
            RR(dst) => {
                let a = self.get_u8(dst);
                let b0 = a & 1;
                let r = (a >> 1) | (self.registers.cy_flag() << 7);
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b0 == 1);

            }
            SLA(dst) => {
                let a = self.get_u8(dst);
                let b7 = (a >> 7) & 1;
                let r = a << 1;
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b7 == 1);
            }
            SRA(dst) => {
                let a = self.get_u8(dst);
                let b0 = a & 1;
                let b7 = (a >> 7) & 1;
                let r = (a >> 1) | (b7 << 7);
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b0 == 1);
            }
            SRL(dst) => {
                let a = self.get_u8(dst);
                let b0 = a & 1;
                let r = a >> 1;
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, b0 == 1);
            }
            SWAP(dst) => {
                let a = self.get_u8(dst);
                let n1 = (a >> 4) & 0xFF;
                let n2 = a & 0xFF;
                let r = (n2 << 4) | n1;
                self.ld8(dst, Src::D8(r));
                self.registers.set_flags(r == 0, false, false, false);
            }
            BIT(bi, src) => {
                let a = self.get_u8(src);
                let b = (a >> bi) & 0x1;
                self.registers.set_flags((!b & 0x1) == 1, false, true, self.registers.cy_flag() == 1);
            }
            RES(bi, dst) => {
                let a = self.get_u8(dst);
                let mask = 0xFF ^ (1 << bi);
                let r = a & mask;
                self.ld8(dst, Src::D8(r));
            }
            SET(bi, dst) => {
                let a = self.get_u8(dst);
                let mask = 1 << bi;
                let r = a | mask;
                self.ld8(dst, Src::D8(r));
            }
            JP(fc, addr) => {
                if self.flag_match(fc) {
                    let r = self.get_u16(addr);
                    self.registers.set16(PC, r);
                    inc_pc = false;
                } else {
                    cycles -= 1;
                }
            }
            JR(fc, offset) => {
                if self.flag_match(fc) {
                    let r = if offset >= 0 {
                        self.registers.get16(PC) + (offset as u16)
                    } else {
                        self.registers.get16(PC) - (offset*-1) as u16
                    };
                    self.registers.set16(PC, r);
                } else {
                    cycles -= 1;
                }
            }
            CALL(fc, addr) => {
                if self.flag_match(fc) {
                    self.push_pc(instr.n_bytes);
                    self.registers.set16(PC, addr);
                    inc_pc = false;
                } else {
                    cycles -= 3;
                }
            }
            RET(fc) => {
                if self.flag_match(fc) {
                    self.pop_pc();
                    inc_pc = false;
                } else {
                    cycles -= 3
                }
            }
            RETI => {
                self.ime = true;
                self.pop_pc();
                inc_pc = false;
            }
            RST(addr) => {
                self.push_pc(instr.n_bytes);
                self.registers.set16(PC, addr as u16);
                inc_pc = false;
            }
            _ => panic!("Not implemented: {:?}", instr.opcode)
        }

        if inc_pc {
            self.registers.inc_pc(instr.n_bytes);
        }

        return cycles;
    }

    fn push_pc(&mut self, n: u16) {
        let pc = self.registers.get16(PC) + n;
        let sp = self.registers.get16(SP);
        let hb = (pc >> 8) as u8;
        let lb = (pc & 0xFF) as u8;

        self.memory_bus.set8(sp-1, hb);
        self.memory_bus.set8(sp-2, lb);
        self.registers.set16(SP, sp-2);
    }

    fn pop_pc(&mut self) {
        let sp = self.registers.get16(SP);
        let lb = self.memory_bus.get8(sp) as u16;
        let hb = self.memory_bus.get8(sp+1) as u16;
        let addr = (hb << 8) | lb;
        self.registers.set16(PC, addr);
        self.registers.set16(SP, sp+2);
    }

    fn flag_match(&self, fc: FlagCondition) -> bool {
        match fc {
            FlagCondition::Z => self.registers.z_flag() == 1,
            FlagCondition::NZ => self.registers.z_flag() == 0,
            FlagCondition::C => self.registers.cy_flag() == 1,
            FlagCondition::NC => self.registers.cy_flag() == 0,
            FlagCondition::ALWAYS => true
        }
    }

    fn ld8(&mut self, dst: Src, src: Src) {
        let v = self.get_u8(src);
        match dst {
            Src::Reg(r) => self.registers.set8(r, v),
            Src::Deref(C) => self.memory_bus.set8(0xFF00 + self.registers.get8(C) as u16, v),
            Src::Deref(r) => self.memory_bus.set8(self.registers.get16(r), v),
            Src::A16(a16) => self.memory_bus.set8(a16, v),
            Src::A8(a8) => self.memory_bus.set8(0xFF00 + a8 as u16, v),
            _ => panic!("Invalid dst for 8 bit value.")
        }
    }

    fn get_u8(&self, src: Src) -> u8 {
        match src {
            Src::Reg(r) => self.registers.get8(r),
            Src::D8(d8) => d8,
            Src::Deref(C) => self.memory_bus.get8(0xFF00 + self.registers.get8(C) as u16),
            Src::Deref(r) => self.memory_bus.get8(self.registers.get16(r)),
            Src::A8(a8) => self.memory_bus.get8(0xFF00 + a8 as u16),
            Src::A16(a16) => self.memory_bus.get8(a16),
            _ => panic!("Invalid src for 8 bit value.")
        }
    }

    fn ld16(&mut self, dst: Src, src: Src) {
        let v = self.get_u16(src);
        match dst {
            Src::Reg(r) => self.registers.set16(r, v),
            Src::A16(a16) => self.memory_bus.set16(a16, v),
            _ => panic!("Invalid src for 16 bit value {:?}", dst)
        }
    }

    fn get_u16(&self, src: Src) -> u16 {
        match src {
            Src::Reg(r) => self.registers.get16(r),
            Src::D16(d16) => d16,
            Src::I8(i8) => i8 as u16,
            _ => panic!("Invalid src for 16 bit value {:?}", src)
        }
    }


    fn disassemble(&self) -> Instr {
        let instr_bytes = self.memory_bus.get_slice(self.registers.pc(), 3);

        Instr::disassemble(instr_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop() {
        let mb = MemoryBus::new_from_slice(&[0x00]);
        let mut cpu = Cpu::new(mb);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
    }

    #[test]
    fn test_02() {
        let mb = MemoryBus::new_from_slice(&[0x02]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(BC, 0x1234);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0x1234), 42);
    }

    #[test]
    fn test_06() {
        let mb = MemoryBus::new_from_slice(&[0x06, 42]);
        let mut cpu = Cpu::new(mb);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(B), 42);
    }

    #[test]
    fn test_0a() {
        let mb = MemoryBus::new_from_slice(&[0x0A, 42]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(BC, 0x1234);
        cpu.memory_bus.set8(0x1234, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 42);
    }

    #[test]
    fn test_e2() {
        let mb = MemoryBus::new_from_slice(&[0xE2]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(C, 0x34);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0xFF34), 42);
    }

    #[test]
    fn test_ea() {
        let mb = MemoryBus::new_from_slice(&[0xEA, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(C, 0x34);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x03);
        assert_eq!(cpu.memory_bus.get8(0xDEAD), 42);
    }

    #[test]
    fn test_f2() {
        let mb = MemoryBus::new_from_slice(&[0xF2]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set8(0xFF34, 42);
        cpu.registers.set8(C, 0x34);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0xFF34), 42);
    }

    #[test]
    fn test_fa() {
        let mb = MemoryBus::new_from_slice(&[0xFA, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set8(0xDEAD, 42);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x03);
        assert_eq!(cpu.registers.get8(A), 42);
    }

    #[test]
    fn test_e0() {
        let mb = MemoryBus::new_from_slice(&[0xE0, 0x34]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0xFF34), 42);
    }

    #[test]
    fn test_f0() {
        let mb = MemoryBus::new_from_slice(&[0xF0, 0x34]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set8(0xFF34, 42);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 42);
    }

    #[test]
    fn test_22() {
        let mb = MemoryBus::new_from_slice(&[0x22]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0xDEAD), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAE);
    }

    #[test]
    fn test_2a() {
        let mb = MemoryBus::new_from_slice(&[0x2a]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.memory_bus.set8(0xDEAD, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAE);
    }

    #[test]
    fn test_32() {
        let mb = MemoryBus::new_from_slice(&[0x32]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0xDEAD), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAC);
    }

    #[test]
    fn test_3a() {
        let mb = MemoryBus::new_from_slice(&[0x3a]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.memory_bus.set8(0xDEAD, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAC);
    }

    #[test]
    fn test_01() {
        let mb = MemoryBus::new_from_slice(&[0x01, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x03);
        assert_eq!(cpu.registers.get16(BC), 0xDEAD);
    }

    #[test]
    fn test_08() {
        let mb = MemoryBus::new_from_slice(&[0x08, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0x1234);

        assert_eq!(cpu.step(), 5);
        assert_eq!(cpu.registers.pc(), 0x03);
        assert_eq!(cpu.memory_bus.get8(0xDEAD), 0x12);
        assert_eq!(cpu.memory_bus.get8(0xDEAE), 0x34);
    }

    #[test]
    fn test_c5() {
        let mb = MemoryBus::new_from_slice(&[0xC5]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFFE);
        cpu.registers.set16(BC, 0x1234);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0xFFFC), 0x12);
        assert_eq!(cpu.memory_bus.get8(0xFFFD), 0x34);
        assert_eq!(cpu.registers.get16(SP), 0xFFFC);
    }

    #[test]
    fn test_e1() {
        let mb = MemoryBus::new_from_slice(&[0xE1]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFFC);
        cpu.memory_bus.set16(0xFFFC, 0xDEAD);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(HL), 0xDEAD);
        assert_eq!(cpu.registers.get16(SP), 0xFFFE);
    }

    #[test]
    fn test_82() {
        let mb = MemoryBus::new_from_slice(&[0x82]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 2);
        cpu.registers.set8(D, 3);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 5);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
        assert_eq!(cpu.registers.get8(F), 0b00000000);
    }

    #[test]
    fn test_82_zhcy() {
        let mb = MemoryBus::new_from_slice(&[0x82]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3A);
        cpu.registers.set8(D, 0xC6);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
        assert_eq!(cpu.registers.get8(F), 0b10110000);
    }

    #[test]
    fn test_82_h() {
        let mb = MemoryBus::new_from_slice(&[0x82]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 10);
        cpu.registers.set8(D, 11);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 21);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_c6_hcy() {
        let mb = MemoryBus::new_from_slice(&[0xC6, 0x7]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 250);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 1);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_8b() {
        let mb = MemoryBus::new_from_slice(&[0x8B]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0xE1);
        cpu.registers.set8(E, 0x0F);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0xF1);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_ce() {
        let mb = MemoryBus::new_from_slice(&[0xCE, 0x3B]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0xE1);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x1D);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_8e() {
        let mb = MemoryBus::new_from_slice(&[0x8E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0xE1);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x1E);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_93() {
        let mb = MemoryBus::new_from_slice(&[0x93]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3E);
        cpu.registers.set8(E, 0x3E);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_d6() {
        let mb = MemoryBus::new_from_slice(&[0xD6, 0x0F]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3E);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x2F);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_96() {
        let mb = MemoryBus::new_from_slice(&[0x96]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3E);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x40);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0xFE);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_9c() {
        let mb = MemoryBus::new_from_slice(&[0x9c]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3B);
        cpu.registers.set8(H, 0x2A);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x10);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_de() {
        let mb = MemoryBus::new_from_slice(&[0xDE, 0x3A]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3B);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_9e() {
        let mb = MemoryBus::new_from_slice(&[0x9E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3B);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x4F);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0xEB);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_a5() {
        let mb = MemoryBus::new_from_slice(&[0xa5]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x5A);
        cpu.registers.set8(L, 0x3F);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x1A);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_f6() {
        let mb = MemoryBus::new_from_slice(&[0xF6, 0x03]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x5A);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x5B);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_ae() {
        let mb = MemoryBus::new_from_slice(&[0xae]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0xFF);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x8A);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x75);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_b8() {
        let mb = MemoryBus::new_from_slice(&[0xB8]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3C);
        cpu.registers.set8(B, 0x2F);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x3C);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_fe() {
        let mb = MemoryBus::new_from_slice(&[0xFE, 0x3C]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3C);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x3C);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_be() {
        let mb = MemoryBus::new_from_slice(&[0xBE]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3C);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x40);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x3C);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_3c() {
        let mb = MemoryBus::new_from_slice(&[0x3C]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0xFF);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_34() {
        let mb = MemoryBus::new_from_slice(&[0x34]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x50);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x51);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_2d() {
        let mb = MemoryBus::new_from_slice(&[0x2D]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(L, 0x01);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(L), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_35() {
        let mb = MemoryBus::new_from_slice(&[0x35]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x00);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0xFF);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_09() {
        let mb = MemoryBus::new_from_slice(&[0x09]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x8A23);
        cpu.registers.set16(BC, 0x0605);
        cpu.registers.set_flags(true, false, false, false);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(HL), 0x9028);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_29() {
        let mb = MemoryBus::new_from_slice(&[0x29]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x8A23);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(HL), 0x1446);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_e8() {
        let mb = MemoryBus::new_from_slice(&[0xE8, 0x02]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFF8);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(SP), 0xFFFA);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_e8_neg() {
        let mb = MemoryBus::new_from_slice(&[0xE8, 0xFE]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFF8);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(SP), 0xFFF6);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_13() {
        let mb = MemoryBus::new_from_slice(&[0x13]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(DE, 0x235F);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(DE), 0x2360);
    }

    #[test]
    fn test_1b() {
        let mb = MemoryBus::new_from_slice(&[0x1B]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(DE, 0x235F);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get16(DE), 0x235E);
    }

    #[test]
    fn test_07() {
        let mb = MemoryBus::new_from_slice(&[0x07]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x85);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        // example in gb programming manual is wrong?
        assert_eq!(cpu.registers.get8(A), 0x0B);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_17() {
        let mb = MemoryBus::new_from_slice(&[0x17]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x95);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x2B);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_0f() {
        let mb = MemoryBus::new_from_slice(&[0x0F]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x3B);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x9D);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_1f() {
        let mb = MemoryBus::new_from_slice(&[0x1F]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x81);
        cpu.registers.set_flags(false, false, false, false);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 0x40);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb00() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x00]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(B, 0x85);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(B), 0x0B);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb06() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x06]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_cb15() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x15]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(L, 0x80);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(L), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb16() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x16]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x11);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x22);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_cb09() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x09]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(C, 0x01);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(C), 0x80);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }


    #[test]
    fn test_cb1f() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x1F]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x01);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb22() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x22]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(D, 0x80);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(D), 0x00);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb2e() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x2E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x8A);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0xC5);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_cb36() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x36]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0xF0);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x0F);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 0);
    }

    #[test]
    fn test_cb3e() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x3E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0xFF);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x7F);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb7f() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x7F]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x80);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb65() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x65]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(L, 0xEF);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb46() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x46]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0xFE);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.z_flag(), 1);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cb4e() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x4E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0xFE);
        cpu.registers.set_flags(false, false, false, true);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.z_flag(), 0);
        assert_eq!(cpu.registers.h_flag(), 1);
        assert_eq!(cpu.registers.n_flag(), 0);
        assert_eq!(cpu.registers.cy_flag(), 1);
    }

    #[test]
    fn test_cbbf() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0xBF]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(A, 0x80);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(A), 0x00);
    }

    #[test]
    fn test_cb93() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0x9E]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0xFF);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0xF7);
    }

    #[test]
    fn test_cbfd() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0xFD]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(L, 0x3B);

        assert_eq!(cpu.step(), 2);
        // example in gb programming manual is wrong?
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.registers.get8(L), 0xBB);
    }

    #[test]
    fn test_cbde() {
        let mb = MemoryBus::new_from_slice(&[0xCB, 0xDE]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x1234);
        cpu.memory_bus.set8(0x1234, 0x00);

        assert_eq!(cpu.step(), 4);
        // example in gb programming manual is wrong?
        assert_eq!(cpu.registers.pc(), 0x02);
        assert_eq!(cpu.memory_bus.get8(0x1234), 0x08);
    }

    #[test]
    fn test_c3() {
        let mb = MemoryBus::new_from_slice(&[0xC3, 0x12, 0x80]);
        let mut cpu = Cpu::new(mb);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x8012);
    }

    #[test]
    fn test_e9() {
        let mb = MemoryBus::new_from_slice(&[0xE9]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0x8012);

        assert_eq!(cpu.step(), 1);
        assert_eq!(cpu.registers.pc(), 0x8012);
    }

    #[test]
    fn test_c2() {
        let mb = MemoryBus::new_from_slice(&[0xC2, 0x00, 0x80]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set_flags(true, false, false, false);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x03);
    }

    #[test]
    fn test_ca() {
        let mb = MemoryBus::new_from_slice(&[0xCA, 0x00, 0x80]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set_flags(true, false, false, false);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x8000);
    }

    #[test]
    fn test_d2() {
        let mb = MemoryBus::new_from_slice(&[0xD2, 0x00, 0x80]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set_flags(true, false, false, false);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x8000);
    }

    #[test]
    fn test_da() {
        let mb = MemoryBus::new_from_slice(&[0xDA, 0x00, 0x80]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set_flags(true, false, false, false);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x03);
    }

    #[test]
    fn test_18() {
        let mb = MemoryBus::new_from_slice(&[0x18, 0x03]);
        let mut cpu = Cpu::new(mb);

        assert_eq!(cpu.step(), 3);
        assert_eq!(cpu.registers.pc(), 0x05);
    }

    #[test]
    fn test_28() {
        let mb = MemoryBus::new_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x28, 0xFD]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set_flags(true, false, false, false);
        cpu.registers.set16(PC, 4);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x03);
    }

    #[test]
    fn test_cd() {
        let mb = MemoryBus::new_from_slice(&[0x00, 0x00, 0x00, 0xcd, 0x34, 0x12]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFFE);
        cpu.registers.set16(PC, 0x03);

        assert_eq!(cpu.step(), 6);
        assert_eq!(cpu.registers.pc(), 0x1234);
        let sp = cpu.registers.get16(SP);
        assert_eq!(sp, 0xFFFC);
        assert_eq!(cpu.memory_bus.get8(sp), 0x06);
        assert_eq!(cpu.memory_bus.get8(sp + 1), 0x00);
    }

    #[test]
    fn test_c9() {
        let mb = MemoryBus::new_from_slice(&[0xC9]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set8(0xFFFC, 0x02);
        cpu.memory_bus.set8(0xFFFD, 0x00);
        cpu.registers.set16(SP, 0xFFFC);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        let sp = cpu.registers.get16(SP);
        assert_eq!(sp, 0xFFFE);
    }

    #[test]
    fn test_d9() {
        let mb = MemoryBus::new_from_slice(&[0xD9]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set8(0xFFFC, 0x02);
        cpu.memory_bus.set8(0xFFFD, 0x00);
        cpu.registers.set16(SP, 0xFFFC);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x02);
        let sp = cpu.registers.get16(SP);
        assert_eq!(sp, 0xFFFE);
        assert!(cpu.ime);
    }

    #[test]
    fn test_e7() {
        let mb = MemoryBus::new_from_slice(&[0x00, 0x00, 0x00, 0xe7]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(SP, 0xFFFE);
        cpu.registers.set16(PC, 0x03);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x20);
        let sp = cpu.registers.get16(SP);
        assert_eq!(sp, 0xFFFC);
        assert_eq!(cpu.memory_bus.get8(sp), 0x04);
        assert_eq!(cpu.memory_bus.get8(sp + 1), 0x00);
    }
}
