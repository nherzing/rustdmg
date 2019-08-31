use crate::registers::{Register};

use Register::*;

#[derive(Debug)]
pub enum Dst {
    Reg(Register),
    Deref(Register),
    A8(u8),
    A16(u16)
}

#[derive(Debug)]
pub enum Val8 {
    Reg(Register),
    Deref(Register),
    A8(u8),
    A16(u16),
    D8(u8)
}

#[derive(Debug)]
pub enum Val16 {
    Reg(Register),
    D16(u16)
}

#[derive(Debug)]
pub enum Opcode {
    NOP,
    LD8(Dst, Val8),
    LD8I(Dst, Val8),
    LD8D(Dst, Val8),
    LD16(Dst, Val16),
    HALT,
}

use Opcode::*;

pub struct Instr {
    pub opcode: Opcode,
    pub cycles: u8,
    pub n_bytes: u16
}

fn n(opcode: Opcode, cycles: u8, n_bytes: u16) -> Instr {
    Instr { opcode, cycles, n_bytes }
}

fn a16(b1: u8, b2: u8) -> u16 {
    ((b1 as u16) << 8) | (b2 as u16)
}

impl Instr {
    pub fn disassemble(bytes: [u8; 3]) -> Instr {
        match bytes[0] {
            0x00 => n(NOP, 1, 1),
            0x02 => n(LD8(Dst::Deref(BC), Val8::Reg(A)), 2, 1),
            0x06 => n(LD8(Dst::Reg(B), Val8::D8(bytes[1])), 2, 2),
            0x0A => n(LD8(Dst::Reg(A), Val8::Deref(BC)), 2, 1),
            0x0E => n(LD8(Dst::Reg(C), Val8::D8(bytes[1])), 2, 2),
            0x12 => n(LD8(Dst::Deref(DE), Val8::Reg(A)), 2, 1),
            0x16 => n(LD8(Dst::Reg(D), Val8::D8(bytes[1])), 2, 2),
            0x1A => n(LD8(Dst::Reg(A), Val8::Deref(DE)), 2, 1),
            0x1E => n(LD8(Dst::Reg(E), Val8::D8(bytes[1])), 2, 2),
            0x22 => n(LD8I(Dst::Deref(HL), Val8::Reg(A)), 2, 1),
            0x26 => n(LD8(Dst::Reg(H), Val8::D8(bytes[1])), 2, 2),
            0x2A => n(LD8I(Dst::Reg(A), Val8::Deref(HL)), 2, 1),
            0x2E => n(LD8(Dst::Reg(L), Val8::D8(bytes[1])), 2, 2),
            0x32 => n(LD8D(Dst::Deref(HL), Val8::Reg(A)), 2, 1),
            0x36 => n(LD8(Dst::Deref(HL), Val8::D8(bytes[1])), 2, 2),
            0x3A => n(LD8D(Dst::Reg(A), Val8::Deref(HL)), 2, 1),
            0x3E => n(LD8(Dst::Reg(A), Val8::D8(bytes[1])), 2, 2),
            0x40 => n(LD8(Dst::Reg(B), Val8::Reg(B)), 1, 1),
            0x41 => n(LD8(Dst::Reg(B), Val8::Reg(C)), 1, 1),
            0x42 => n(LD8(Dst::Reg(B), Val8::Reg(D)), 1, 1),
            0x43 => n(LD8(Dst::Reg(B), Val8::Reg(E)), 1, 1),
            0x44 => n(LD8(Dst::Reg(B), Val8::Reg(H)), 1, 1),
            0x45 => n(LD8(Dst::Reg(B), Val8::Reg(L)), 1, 1),
            0x46 => n(LD8(Dst::Reg(B), Val8::Deref(HL)), 2, 1),
            0x47 => n(LD8(Dst::Reg(B), Val8::Reg(A)), 1, 1),
            0x48 => n(LD8(Dst::Reg(C), Val8::Reg(B)), 1, 1),
            0x49 => n(LD8(Dst::Reg(C), Val8::Reg(C)), 1, 1),
            0x4A => n(LD8(Dst::Reg(C), Val8::Reg(D)), 1, 1),
            0x4B => n(LD8(Dst::Reg(C), Val8::Reg(E)), 1, 1),
            0x4C => n(LD8(Dst::Reg(C), Val8::Reg(H)), 1, 1),
            0x4D => n(LD8(Dst::Reg(C), Val8::Reg(L)), 1, 1),
            0x4E => n(LD8(Dst::Reg(C), Val8::Deref(HL)), 2, 1),
            0x4F => n(LD8(Dst::Reg(C), Val8::Reg(A)), 1, 1),
            0x50 => n(LD8(Dst::Reg(D), Val8::Reg(B)), 1, 1),
            0x51 => n(LD8(Dst::Reg(D), Val8::Reg(C)), 1, 1),
            0x52 => n(LD8(Dst::Reg(D), Val8::Reg(D)), 1, 1),
            0x53 => n(LD8(Dst::Reg(D), Val8::Reg(E)), 1, 1),
            0x54 => n(LD8(Dst::Reg(D), Val8::Reg(H)), 1, 1),
            0x55 => n(LD8(Dst::Reg(D), Val8::Reg(L)), 1, 1),
            0x56 => n(LD8(Dst::Reg(D), Val8::Deref(HL)), 2, 1),
            0x57 => n(LD8(Dst::Reg(D), Val8::Reg(A)), 1, 1),
            0x58 => n(LD8(Dst::Reg(E), Val8::Reg(B)), 1, 1),
            0x59 => n(LD8(Dst::Reg(E), Val8::Reg(C)), 1, 1),
            0x5A => n(LD8(Dst::Reg(E), Val8::Reg(D)), 1, 1),
            0x5B => n(LD8(Dst::Reg(E), Val8::Reg(E)), 1, 1),
            0x5C => n(LD8(Dst::Reg(E), Val8::Reg(H)), 1, 1),
            0x5D => n(LD8(Dst::Reg(E), Val8::Reg(L)), 1, 1),
            0x5E => n(LD8(Dst::Reg(E), Val8::Deref(HL)), 2, 1),
            0x5F => n(LD8(Dst::Reg(E), Val8::Reg(A)), 1, 1),
            0x60 => n(LD8(Dst::Reg(H), Val8::Reg(B)), 1, 1),
            0x61 => n(LD8(Dst::Reg(H), Val8::Reg(C)), 1, 1),
            0x62 => n(LD8(Dst::Reg(H), Val8::Reg(D)), 1, 1),
            0x63 => n(LD8(Dst::Reg(H), Val8::Reg(E)), 1, 1),
            0x64 => n(LD8(Dst::Reg(H), Val8::Reg(H)), 1, 1),
            0x65 => n(LD8(Dst::Reg(H), Val8::Reg(L)), 1, 1),
            0x66 => n(LD8(Dst::Reg(H), Val8::Deref(HL)), 2, 1),
            0x67 => n(LD8(Dst::Reg(H), Val8::Reg(A)), 1, 1),
            0x68 => n(LD8(Dst::Reg(L), Val8::Reg(B)), 1, 1),
            0x69 => n(LD8(Dst::Reg(L), Val8::Reg(C)), 1, 1),
            0x6A => n(LD8(Dst::Reg(L), Val8::Reg(D)), 1, 1),
            0x6B => n(LD8(Dst::Reg(L), Val8::Reg(E)), 1, 1),
            0x6C => n(LD8(Dst::Reg(L), Val8::Reg(H)), 1, 1),
            0x6D => n(LD8(Dst::Reg(L), Val8::Reg(L)), 1, 1),
            0x6E => n(LD8(Dst::Reg(L), Val8::Deref(HL)), 2, 1),
            0x6F => n(LD8(Dst::Reg(L), Val8::Reg(A)), 1, 1),
            0x70 => n(LD8(Dst::Deref(HL), Val8::Reg(B)), 1, 1),
            0x71 => n(LD8(Dst::Deref(HL), Val8::Reg(C)), 1, 1),
            0x72 => n(LD8(Dst::Deref(HL), Val8::Reg(D)), 1, 1),
            0x73 => n(LD8(Dst::Deref(HL), Val8::Reg(E)), 1, 1),
            0x74 => n(LD8(Dst::Deref(HL), Val8::Reg(H)), 1, 1),
            0x75 => n(LD8(Dst::Deref(HL), Val8::Reg(L)), 1, 1),
            0x76 => n(HALT, 1, 1),
            0x77 => n(LD8(Dst::Deref(HL), Val8::Reg(A)), 1, 1),
            0x78 => n(LD8(Dst::Reg(A), Val8::Reg(B)), 1, 1),
            0x79 => n(LD8(Dst::Reg(A), Val8::Reg(C)), 1, 1),
            0x7A => n(LD8(Dst::Reg(A), Val8::Reg(D)), 1, 1),
            0x7B => n(LD8(Dst::Reg(A), Val8::Reg(E)), 1, 1),
            0x7C => n(LD8(Dst::Reg(A), Val8::Reg(H)), 1, 1),
            0x7D => n(LD8(Dst::Reg(A), Val8::Reg(L)), 1, 1),
            0x7E => n(LD8(Dst::Reg(A), Val8::Deref(HL)), 2, 1),
            0x7F => n(LD8(Dst::Reg(A), Val8::Reg(A)), 1, 1),
            0xE0 => n(LD8(Dst::A8(bytes[1]), Val8::Reg(A)), 3, 2),
            0xE2 => n(LD8(Dst::Deref(C), Val8::Reg(A)), 2, 1),
            0xEA => n(LD8(Dst::A16(a16(bytes[1], bytes[2])), Val8::Reg(A)), 4, 3),
            0xF0 => n(LD8(Dst::Reg(A), Val8::A8(bytes[1])), 3, 2),
            0xF2 => n(LD8(Dst::Reg(A), Val8::Deref(C)), 2, 1),
            0xFA => n(LD8(Dst::Reg(A), Val8::A16(a16(bytes[1], bytes[2]))), 4, 3),
                // n(opcodes, cycles, bytes)
            _ => panic!("Unknown opcode")
        }
    }
}
