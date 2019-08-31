use crate::memory_bus::{MemoryBus};
use crate::registers::{Register, Registers};
use crate::opcode::{Dst, Val8, Val16, Opcode, Instr};

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
            _ => panic!("Not implemented: {:?}", instr.opcode)
        }

        self.registers.inc_pc(instr.n_bytes);
        return instr.cycles;
    }

    fn ld8(&mut self, dst: Dst, src: Val8) {
        let v = self.get_u8(src);
        match dst {
            Dst::Reg(r) => self.registers.set8(r, v),
            Dst::Deref(C) => self.memory_bus.set_byte(0xFF00 + self.registers.get8(C) as u16, v),
            Dst::Deref(r) => self.memory_bus.set_byte(self.registers.get16(r), v),
            Dst::A16(a16) => self.memory_bus.set_byte(a16, v),
            Dst::A8(a8) => self.memory_bus.set_byte(0xFF00 + a8 as u16, v)
        }
    }

    fn get_u8(&self, val8: Val8) -> u8 {
        match val8 {
            Val8::Reg(r) => self.registers.get8(r),
            Val8::D8(d8) => d8,
            Val8::Deref(C) => self.memory_bus.read_byte(0xFF00 + self.registers.get8(C) as u16),
            Val8::Deref(r) => self.memory_bus.read_byte(self.registers.get16(r)),
            Val8::A8(a8) => self.memory_bus.read_byte(0xFF00 + a8 as u16),
            Val8::A16(a16) => self.memory_bus.read_byte(a16)
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
        assert_eq!(cpu.memory_bus.read_byte(0x1234), 42);
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
        cpu.memory_bus.set_byte(0x1234, 42);

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
        assert_eq!(cpu.memory_bus.read_byte(0xFF34), 42);
    }

    #[test]
    fn test_ea() {
        let mb = MemoryBus::new_from_slice(&[0xEA, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set8(C, 0x34);
        cpu.registers.set8(A, 42);

        assert_eq!(cpu.step(), 4);
        assert_eq!(cpu.registers.pc(), 0x03);
        assert_eq!(cpu.memory_bus.read_byte(0xDEAD), 42);
    }

    #[test]
    fn test_f2() {
        let mb = MemoryBus::new_from_slice(&[0xF2]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set_byte(0xFF34, 42);
        cpu.registers.set8(C, 0x34);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.memory_bus.read_byte(0xFF34), 42);
    }

    #[test]
    fn test_fa() {
        let mb = MemoryBus::new_from_slice(&[0xFA, 0xDE, 0xAD]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set_byte(0xDEAD, 42);

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
        assert_eq!(cpu.memory_bus.read_byte(0xFF34), 42);
    }

    #[test]
    fn test_f0() {
        let mb = MemoryBus::new_from_slice(&[0xF0, 0x34]);
        let mut cpu = Cpu::new(mb);
        cpu.memory_bus.set_byte(0xFF34, 42);

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
        assert_eq!(cpu.memory_bus.read_byte(0xDEAD), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAE);
    }

    #[test]
    fn test_2a() {
        let mb = MemoryBus::new_from_slice(&[0x2a]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.memory_bus.set_byte(0xDEAD, 42);

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
        assert_eq!(cpu.memory_bus.read_byte(0xDEAD), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAC);
    }

    #[test]
    fn test_3a() {
        let mb = MemoryBus::new_from_slice(&[0x3a]);
        let mut cpu = Cpu::new(mb);
        cpu.registers.set16(HL, 0xDEAD);
        cpu.memory_bus.set_byte(0xDEAD, 42);

        assert_eq!(cpu.step(), 2);
        assert_eq!(cpu.registers.pc(), 0x01);
        assert_eq!(cpu.registers.get8(A), 42);
        assert_eq!(cpu.registers.get16(HL), 0xDEAC);
    }

}
