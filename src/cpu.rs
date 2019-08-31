use crate::memory_bus::{MemoryBus};
use crate::registers::{Register, Registers};
use crate::opcode::{Dst, Src, Opcode, Instr};

use Opcode::*;
use Register::*;

struct Cpu {
    memory_bus: MemoryBus,
    registers: Registers
}

impl Cpu {
    pub fn new(memory_bus: MemoryBus) -> Cpu {
        Cpu {
            memory_bus,
            registers: Registers::new()
        }
    }

    pub fn step(&mut self) -> u8 {
        let instr = self.disassemble();

        match instr.opcode {
            NOP => {}
            HALT => {}
            LD8(dst, src) => {
                self.ld8(dst, src);
            },
            LD8I(dst, src) => {
                self.ld8(dst, src);
                self.registers.set16(HL, self.registers.get16(HL) + 1);
            },
            LD8D(dst, src) => {
                self.ld8(dst, src);
                self.registers.set16(HL, self.registers.get16(HL) - 1);
            },
            LD16(dst, src) => {
                self.ld16(dst, src);
            },
            PUSH(reg) => {
                let sp = self.registers.get16(SP);
                self.registers.set16(SP, sp - 2);
                self.memory_bus.set16(sp - 2, self.registers.get16(reg));
            },
            POP(reg) => {
                let sp = self.registers.get16(SP);
                self.registers.set16(SP, sp + 2);
                self.registers.set16(reg, self.memory_bus.get16(sp));
            },
            _ => panic!("Not implemented: {:?}", instr.opcode)
        }

        self.registers.inc_pc(instr.n_bytes);
        return instr.cycles;
    }

    fn ld8(&mut self, dst: Dst, src: Src) {
        let v = self.get_u8(src);
        match dst {
            Dst::Reg(r) => self.registers.set8(r, v),
            Dst::Deref(C) => self.memory_bus.set8(0xFF00 + self.registers.get8(C) as u16, v),
            Dst::Deref(r) => self.memory_bus.set8(self.registers.get16(r), v),
            Dst::A16(a16) => self.memory_bus.set8(a16, v),
            Dst::A8(a8) => self.memory_bus.set8(0xFF00 + a8 as u16, v)
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
            Src::D16(_) => panic!("Can't use d16 as 8 bit value.")
        }
    }

    fn ld16(&mut self, dst: Dst, src: Src) {
        let v = self.get_u16(src);
        match dst {
            Dst::Reg(r) => self.registers.set16(r, v),
            Dst::A16(a16) => self.memory_bus.set16(a16, v),
            _ => panic!("Invalid dst for 16 bit value {:?}", dst)
        }
    }

    fn get_u16(&self, src: Src) -> u16 {
        match src {
            Src::Reg(r) => self.registers.get16(r),
            Src::D16(d16) => d16,
            _ => panic!("Invalid src for 16 bit value {:?}", src)
        }
    }


    fn disassemble(&self) -> Instr {
        let mut instr_bytes: [u8; 3] = [0; 3];
        self.memory_bus.read_bytes(self.registers.pc(), &mut instr_bytes);

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

}
