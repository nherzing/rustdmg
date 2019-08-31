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
pub enum Src {
    Reg(Register),
    Deref(Register),
    A8(u8),
    A16(u16),
    D8(u8),
    D16(u16)
}

#[derive(Debug)]
pub enum Opcode {
    NOP,
    LD8(Dst, Src),
    LD8I(Dst, Src),
    LD8D(Dst, Src),
    LD16(Dst, Src),
    PUSH(Register),
    POP(Register),
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

impl Instr {
    pub fn disassemble(bytes: [u8; 3]) -> Instr {
        let b1 = bytes[1];
        let b12 = ((bytes[1] as u16) << 8) | (bytes[2] as u16);

        match bytes[0] {
            0x00 => n(NOP, 1, 1),
            0x01 => n(LD16(Dst::Reg(BC), Src::D16(b12)), 3, 3),
            0x02 => n(LD8(Dst::Deref(BC), Src::Reg(A)), 2, 1),
            0x06 => n(LD8(Dst::Reg(B), Src::D8(b1)), 2, 2),
            0x08 => n(LD16(Dst::A16(b12), Src::Reg(SP)), 5, 3),
            0x0A => n(LD8(Dst::Reg(A), Src::Deref(BC)), 2, 1),
            0x0E => n(LD8(Dst::Reg(C), Src::D8(b1)), 2, 2),
            0x11 => n(LD16(Dst::Reg(DE), Src::D16(b12)), 3, 3),
            0x12 => n(LD8(Dst::Deref(DE), Src::Reg(A)), 2, 1),
            0x16 => n(LD8(Dst::Reg(D), Src::D8(b1)), 2, 2),
            0x1A => n(LD8(Dst::Reg(A), Src::Deref(DE)), 2, 1),
            0x1E => n(LD8(Dst::Reg(E), Src::D8(b1)), 2, 2),
            0x21 => n(LD16(Dst::Reg(HL), Src::D16(b12)), 3, 3),
            0x22 => n(LD8I(Dst::Deref(HL), Src::Reg(A)), 2, 1),
            0x26 => n(LD8(Dst::Reg(H), Src::D8(b1)), 2, 2),
            0x2A => n(LD8I(Dst::Reg(A), Src::Deref(HL)), 2, 1),
            0x2E => n(LD8(Dst::Reg(L), Src::D8(b1)), 2, 2),
            0x31 => n(LD16(Dst::Reg(SP), Src::D16(b12)), 3, 3),
            0x32 => n(LD8D(Dst::Deref(HL), Src::Reg(A)), 2, 1),
            0x36 => n(LD8(Dst::Deref(HL), Src::D8(b1)), 2, 2),
            0x3A => n(LD8D(Dst::Reg(A), Src::Deref(HL)), 2, 1),
            0x3E => n(LD8(Dst::Reg(A), Src::D8(b1)), 2, 2),
            0x40 => n(LD8(Dst::Reg(B), Src::Reg(B)), 1, 1),
            0x41 => n(LD8(Dst::Reg(B), Src::Reg(C)), 1, 1),
            0x42 => n(LD8(Dst::Reg(B), Src::Reg(D)), 1, 1),
            0x43 => n(LD8(Dst::Reg(B), Src::Reg(E)), 1, 1),
            0x44 => n(LD8(Dst::Reg(B), Src::Reg(H)), 1, 1),
            0x45 => n(LD8(Dst::Reg(B), Src::Reg(L)), 1, 1),
            0x46 => n(LD8(Dst::Reg(B), Src::Deref(HL)), 2, 1),
            0x47 => n(LD8(Dst::Reg(B), Src::Reg(A)), 1, 1),
            0x48 => n(LD8(Dst::Reg(C), Src::Reg(B)), 1, 1),
            0x49 => n(LD8(Dst::Reg(C), Src::Reg(C)), 1, 1),
            0x4A => n(LD8(Dst::Reg(C), Src::Reg(D)), 1, 1),
            0x4B => n(LD8(Dst::Reg(C), Src::Reg(E)), 1, 1),
            0x4C => n(LD8(Dst::Reg(C), Src::Reg(H)), 1, 1),
            0x4D => n(LD8(Dst::Reg(C), Src::Reg(L)), 1, 1),
            0x4E => n(LD8(Dst::Reg(C), Src::Deref(HL)), 2, 1),
            0x4F => n(LD8(Dst::Reg(C), Src::Reg(A)), 1, 1),
            0x50 => n(LD8(Dst::Reg(D), Src::Reg(B)), 1, 1),
            0x51 => n(LD8(Dst::Reg(D), Src::Reg(C)), 1, 1),
            0x52 => n(LD8(Dst::Reg(D), Src::Reg(D)), 1, 1),
            0x53 => n(LD8(Dst::Reg(D), Src::Reg(E)), 1, 1),
            0x54 => n(LD8(Dst::Reg(D), Src::Reg(H)), 1, 1),
            0x55 => n(LD8(Dst::Reg(D), Src::Reg(L)), 1, 1),
            0x56 => n(LD8(Dst::Reg(D), Src::Deref(HL)), 2, 1),
            0x57 => n(LD8(Dst::Reg(D), Src::Reg(A)), 1, 1),
            0x58 => n(LD8(Dst::Reg(E), Src::Reg(B)), 1, 1),
            0x59 => n(LD8(Dst::Reg(E), Src::Reg(C)), 1, 1),
            0x5A => n(LD8(Dst::Reg(E), Src::Reg(D)), 1, 1),
            0x5B => n(LD8(Dst::Reg(E), Src::Reg(E)), 1, 1),
            0x5C => n(LD8(Dst::Reg(E), Src::Reg(H)), 1, 1),
            0x5D => n(LD8(Dst::Reg(E), Src::Reg(L)), 1, 1),
            0x5E => n(LD8(Dst::Reg(E), Src::Deref(HL)), 2, 1),
            0x5F => n(LD8(Dst::Reg(E), Src::Reg(A)), 1, 1),
            0x60 => n(LD8(Dst::Reg(H), Src::Reg(B)), 1, 1),
            0x61 => n(LD8(Dst::Reg(H), Src::Reg(C)), 1, 1),
            0x62 => n(LD8(Dst::Reg(H), Src::Reg(D)), 1, 1),
            0x63 => n(LD8(Dst::Reg(H), Src::Reg(E)), 1, 1),
            0x64 => n(LD8(Dst::Reg(H), Src::Reg(H)), 1, 1),
            0x65 => n(LD8(Dst::Reg(H), Src::Reg(L)), 1, 1),
            0x66 => n(LD8(Dst::Reg(H), Src::Deref(HL)), 2, 1),
            0x67 => n(LD8(Dst::Reg(H), Src::Reg(A)), 1, 1),
            0x68 => n(LD8(Dst::Reg(L), Src::Reg(B)), 1, 1),
            0x69 => n(LD8(Dst::Reg(L), Src::Reg(C)), 1, 1),
            0x6A => n(LD8(Dst::Reg(L), Src::Reg(D)), 1, 1),
            0x6B => n(LD8(Dst::Reg(L), Src::Reg(E)), 1, 1),
            0x6C => n(LD8(Dst::Reg(L), Src::Reg(H)), 1, 1),
            0x6D => n(LD8(Dst::Reg(L), Src::Reg(L)), 1, 1),
            0x6E => n(LD8(Dst::Reg(L), Src::Deref(HL)), 2, 1),
            0x6F => n(LD8(Dst::Reg(L), Src::Reg(A)), 1, 1),
            0x70 => n(LD8(Dst::Deref(HL), Src::Reg(B)), 1, 1),
            0x71 => n(LD8(Dst::Deref(HL), Src::Reg(C)), 1, 1),
            0x72 => n(LD8(Dst::Deref(HL), Src::Reg(D)), 1, 1),
            0x73 => n(LD8(Dst::Deref(HL), Src::Reg(E)), 1, 1),
            0x74 => n(LD8(Dst::Deref(HL), Src::Reg(H)), 1, 1),
            0x75 => n(LD8(Dst::Deref(HL), Src::Reg(L)), 1, 1),
            0x76 => n(HALT, 1, 1),
            0x77 => n(LD8(Dst::Deref(HL), Src::Reg(A)), 1, 1),
            0x78 => n(LD8(Dst::Reg(A), Src::Reg(B)), 1, 1),
            0x79 => n(LD8(Dst::Reg(A), Src::Reg(C)), 1, 1),
            0x7A => n(LD8(Dst::Reg(A), Src::Reg(D)), 1, 1),
            0x7B => n(LD8(Dst::Reg(A), Src::Reg(E)), 1, 1),
            0x7C => n(LD8(Dst::Reg(A), Src::Reg(H)), 1, 1),
            0x7D => n(LD8(Dst::Reg(A), Src::Reg(L)), 1, 1),
            0x7E => n(LD8(Dst::Reg(A), Src::Deref(HL)), 2, 1),
            0x7F => n(LD8(Dst::Reg(A), Src::Reg(A)), 1, 1),
            0xC1 => n(POP(BC), 4, 1),
            0xC5 => n(PUSH(BC), 4, 1),
            0xD1 => n(POP(DE), 4, 1),
            0xD5 => n(PUSH(DE), 4, 1),
            0xE0 => n(LD8(Dst::A8(b1), Src::Reg(A)), 3, 2),
            0xE1 => n(POP(HL), 4, 1),
            0xE5 => n(PUSH(HL), 4, 1),
            0xE2 => n(LD8(Dst::Deref(C), Src::Reg(A)), 2, 1),
            0xEA => n(LD8(Dst::A16(b12), Src::Reg(A)), 4, 3),
            0xF0 => n(LD8(Dst::Reg(A), Src::A8(bytes[1])), 3, 2),
            0xF1 => n(POP(AF), 4, 1),
            0xF2 => n(LD8(Dst::Reg(A), Src::Deref(C)), 2, 1),
            0xF5 => n(PUSH(AF), 4, 1),
            0xF9 => n(LD16(Dst::Reg(SP), Src::Reg(HL)), 2, 1),
            0xFA => n(LD8(Dst::Reg(A), Src::A16(b12)), 4, 3),
                // n(opcodes, cycles, bytes)
            _ => panic!("Unknown opcode")
        }
    }
}
