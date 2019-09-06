use super::registers::{Register};

use Register::*;

#[derive(Copy, Clone, Debug)]
pub enum Src {
    Reg(Register),
    Deref(Register),
    A8(u8),
    A16(u16),
    D8(u8),
    D16(u16),
    I8(i8),
}

#[derive(Copy, Clone, Debug)]
pub enum FlagCondition {
    Z,
    NZ,
    C,
    NC,
    ALWAYS
}

#[derive(Copy, Clone, Debug)]
pub enum Opcode {
    NOP,
    LD8(Src, Src),
    LD8I(Src, Src),
    LD8D(Src, Src),
    LD16(Src, Src),
    PUSH(Register),
    POP(Register),
    ADD8(Src),
    ADC8(Src),
    SUB8(Src),
    SBC8(Src),
    AND(Src),
    OR(Src),
    XOR(Src),
    CP(Src),
    INC8(Src),
    DEC8(Src),
    ADD16(Register, Src),
    INC16(Register),
    DEC16(Register),
    RLCA,
    RLA,
    RRCA,
    RRA,
    RLC(Src),
    RL(Src),
    RRC(Src),
    RR(Src),
    SLA(Src),
    SRA(Src),
    SRL(Src),
    SWAP(Src),
    BIT(u8, Src),
    SET(u8, Src),
    RES(u8, Src),
    JP(FlagCondition, Src),
    JR(FlagCondition, i8),
    CALL(FlagCondition, u16),
    RET(FlagCondition),
    RETI,
    RST(u8),
    DAA,
    CPL,
    CCF,
    SCF,
    DI,
    EI,
    HALT,
    STOP,
    INVALID(u8)
}

use Opcode::*;

pub struct Instr {
    pub opcode: Opcode,
    pub cycles: u32,
    pub n_bytes: u16
}

fn n(opcode: Opcode, cycles: u32, n_bytes: u16) -> Instr {
    Instr { opcode, cycles, n_bytes }
}

impl Instr {
    pub fn disassemble(bytes: &[u8]) -> Instr {
        let b0 = bytes[0];
        let b1 = bytes[1];
        let b21 = ((bytes[2] as u16) << 8) | (bytes[1] as u16);

        match b0 {
            0x00 => n(NOP, 1, 1),
            0x01 => n(LD16(Src::Reg(BC), Src::D16(b21)), 3, 3),
            0x02 => n(LD8(Src::Deref(BC), Src::Reg(A)), 2, 1),
            0x03 => n(INC16(BC), 2, 1),
            0x04 => n(INC8(Src::Reg(B)), 1, 1),
            0x05 => n(DEC8(Src::Reg(B)), 1, 1),
            0x06 => n(LD8(Src::Reg(B), Src::D8(b1)), 2, 2),
            0x07 => n(RLCA, 1, 1),
            0x08 => n(LD16(Src::A16(b21), Src::Reg(SP)), 5, 3),
            0x09 => n(ADD16(HL, Src::Reg(BC)), 2, 1),
            0x0A => n(LD8(Src::Reg(A), Src::Deref(BC)), 2, 1),
            0x0B => n(DEC16(BC), 2, 1),
            0x0C => n(INC8(Src::Reg(C)), 1, 1),
            0x0D => n(DEC8(Src::Reg(C)), 1, 1),
            0x0E => n(LD8(Src::Reg(C), Src::D8(b1)), 2, 2),
            0x0F => n(RRCA, 1, 1),
            0x10 => n(STOP, 1, 1),
            0x11 => n(LD16(Src::Reg(DE), Src::D16(b21)), 3, 3),
            0x12 => n(LD8(Src::Deref(DE), Src::Reg(A)), 2, 1),
            0x13 => n(INC16(DE), 2, 1),
            0x14 => n(INC8(Src::Reg(D)), 1, 1),
            0x15 => n(DEC8(Src::Reg(D)), 1, 1),
            0x16 => n(LD8(Src::Reg(D), Src::D8(b1)), 2, 2),
            0x17 => n(RLA, 1, 1),
            0x18 => n(JR(FlagCondition::ALWAYS, b1 as i8), 3, 2),
            0x19 => n(ADD16(HL, Src::Reg(DE)), 2, 1),
            0x1A => n(LD8(Src::Reg(A), Src::Deref(DE)), 2, 1),
            0x1B => n(DEC16(DE), 2, 1),
            0x1C => n(INC8(Src::Reg(E)), 1, 1),
            0x1D => n(DEC8(Src::Reg(E)), 1, 1),
            0x1E => n(LD8(Src::Reg(E), Src::D8(b1)), 2, 2),
            0x1F => n(RRA, 1, 1),
            0x20 => n(JR(FlagCondition::NZ, b1 as i8), 4, 2),
            0x21 => n(LD16(Src::Reg(HL), Src::D16(b21)), 3, 3),
            0x22 => n(LD8I(Src::Deref(HL), Src::Reg(A)), 2, 1),
            0x23 => n(INC16(HL), 2, 1),
            0x24 => n(INC8(Src::Reg(H)), 1, 1),
            0x25 => n(DEC8(Src::Reg(H)), 1, 1),
            0x26 => n(LD8(Src::Reg(H), Src::D8(b1)), 2, 2),
            0x27 => n(DAA, 1, 1),
            0x28 => n(JR(FlagCondition::Z, b1 as i8), 4, 2),
            0x29 => n(ADD16(HL, Src::Reg(HL)), 2, 1),
            0x2A => n(LD8I(Src::Reg(A), Src::Deref(HL)), 2, 1),
            0x2B => n(DEC16(HL), 2, 1),
            0x2C => n(INC8(Src::Reg(L)), 1, 1),
            0x2D => n(DEC8(Src::Reg(L)), 1, 1),
            0x2E => n(LD8(Src::Reg(L), Src::D8(b1)), 2, 2),
            0x2F => n(CPL, 1, 1),
            0x30 => n(JR(FlagCondition::NC, b1 as i8), 4, 2),
            0x31 => n(LD16(Src::Reg(SP), Src::D16(b21)), 3, 3),
            0x32 => n(LD8D(Src::Deref(HL), Src::Reg(A)), 2, 1),
            0x33 => n(INC16(SP), 2, 1),
            0x34 => n(INC8(Src::Deref(HL)), 3, 1),
            0x35 => n(DEC8(Src::Deref(HL)), 3, 1),
            0x36 => n(LD8(Src::Deref(HL), Src::D8(b1)), 2, 2),
            0x37 => n(SCF, 1, 1),
            0x38 => n(JR(FlagCondition::C, b1 as i8), 4, 2),
            0x39 => n(ADD16(HL, Src::Reg(SP)), 2, 1),
            0x3A => n(LD8D(Src::Reg(A), Src::Deref(HL)), 2, 1),
            0x3B => n(DEC16(SP), 2, 1),
            0x3C => n(INC8(Src::Reg(A)), 1, 1),
            0x3D => n(DEC8(Src::Reg(A)), 1, 1),
            0x3E => n(LD8(Src::Reg(A), Src::D8(b1)), 2, 2),
            0x3F => n(CCF, 1, 1),
            0x40 => n(LD8(Src::Reg(B), Src::Reg(B)), 1, 1),
            0x41 => n(LD8(Src::Reg(B), Src::Reg(C)), 1, 1),
            0x42 => n(LD8(Src::Reg(B), Src::Reg(D)), 1, 1),
            0x43 => n(LD8(Src::Reg(B), Src::Reg(E)), 1, 1),
            0x44 => n(LD8(Src::Reg(B), Src::Reg(H)), 1, 1),
            0x45 => n(LD8(Src::Reg(B), Src::Reg(L)), 1, 1),
            0x46 => n(LD8(Src::Reg(B), Src::Deref(HL)), 2, 1),
            0x47 => n(LD8(Src::Reg(B), Src::Reg(A)), 1, 1),
            0x48 => n(LD8(Src::Reg(C), Src::Reg(B)), 1, 1),
            0x49 => n(LD8(Src::Reg(C), Src::Reg(C)), 1, 1),
            0x4A => n(LD8(Src::Reg(C), Src::Reg(D)), 1, 1),
            0x4B => n(LD8(Src::Reg(C), Src::Reg(E)), 1, 1),
            0x4C => n(LD8(Src::Reg(C), Src::Reg(H)), 1, 1),
            0x4D => n(LD8(Src::Reg(C), Src::Reg(L)), 1, 1),
            0x4E => n(LD8(Src::Reg(C), Src::Deref(HL)), 2, 1),
            0x4F => n(LD8(Src::Reg(C), Src::Reg(A)), 1, 1),
            0x50 => n(LD8(Src::Reg(D), Src::Reg(B)), 1, 1),
            0x51 => n(LD8(Src::Reg(D), Src::Reg(C)), 1, 1),
            0x52 => n(LD8(Src::Reg(D), Src::Reg(D)), 1, 1),
            0x53 => n(LD8(Src::Reg(D), Src::Reg(E)), 1, 1),
            0x54 => n(LD8(Src::Reg(D), Src::Reg(H)), 1, 1),
            0x55 => n(LD8(Src::Reg(D), Src::Reg(L)), 1, 1),
            0x56 => n(LD8(Src::Reg(D), Src::Deref(HL)), 2, 1),
            0x57 => n(LD8(Src::Reg(D), Src::Reg(A)), 1, 1),
            0x58 => n(LD8(Src::Reg(E), Src::Reg(B)), 1, 1),
            0x59 => n(LD8(Src::Reg(E), Src::Reg(C)), 1, 1),
            0x5A => n(LD8(Src::Reg(E), Src::Reg(D)), 1, 1),
            0x5B => n(LD8(Src::Reg(E), Src::Reg(E)), 1, 1),
            0x5C => n(LD8(Src::Reg(E), Src::Reg(H)), 1, 1),
            0x5D => n(LD8(Src::Reg(E), Src::Reg(L)), 1, 1),
            0x5E => n(LD8(Src::Reg(E), Src::Deref(HL)), 2, 1),
            0x5F => n(LD8(Src::Reg(E), Src::Reg(A)), 1, 1),
            0x60 => n(LD8(Src::Reg(H), Src::Reg(B)), 1, 1),
            0x61 => n(LD8(Src::Reg(H), Src::Reg(C)), 1, 1),
            0x62 => n(LD8(Src::Reg(H), Src::Reg(D)), 1, 1),
            0x63 => n(LD8(Src::Reg(H), Src::Reg(E)), 1, 1),
            0x64 => n(LD8(Src::Reg(H), Src::Reg(H)), 1, 1),
            0x65 => n(LD8(Src::Reg(H), Src::Reg(L)), 1, 1),
            0x66 => n(LD8(Src::Reg(H), Src::Deref(HL)), 2, 1),
            0x67 => n(LD8(Src::Reg(H), Src::Reg(A)), 1, 1),
            0x68 => n(LD8(Src::Reg(L), Src::Reg(B)), 1, 1),
            0x69 => n(LD8(Src::Reg(L), Src::Reg(C)), 1, 1),
            0x6A => n(LD8(Src::Reg(L), Src::Reg(D)), 1, 1),
            0x6B => n(LD8(Src::Reg(L), Src::Reg(E)), 1, 1),
            0x6C => n(LD8(Src::Reg(L), Src::Reg(H)), 1, 1),
            0x6D => n(LD8(Src::Reg(L), Src::Reg(L)), 1, 1),
            0x6E => n(LD8(Src::Reg(L), Src::Deref(HL)), 2, 1),
            0x6F => n(LD8(Src::Reg(L), Src::Reg(A)), 1, 1),
            0x70 => n(LD8(Src::Deref(HL), Src::Reg(B)), 1, 1),
            0x71 => n(LD8(Src::Deref(HL), Src::Reg(C)), 1, 1),
            0x72 => n(LD8(Src::Deref(HL), Src::Reg(D)), 1, 1),
            0x73 => n(LD8(Src::Deref(HL), Src::Reg(E)), 1, 1),
            0x74 => n(LD8(Src::Deref(HL), Src::Reg(H)), 1, 1),
            0x75 => n(LD8(Src::Deref(HL), Src::Reg(L)), 1, 1),
            0x76 => n(HALT, 1, 1),
            0x77 => n(LD8(Src::Deref(HL), Src::Reg(A)), 1, 1),
            0x78 => n(LD8(Src::Reg(A), Src::Reg(B)), 1, 1),
            0x79 => n(LD8(Src::Reg(A), Src::Reg(C)), 1, 1),
            0x7A => n(LD8(Src::Reg(A), Src::Reg(D)), 1, 1),
            0x7B => n(LD8(Src::Reg(A), Src::Reg(E)), 1, 1),
            0x7C => n(LD8(Src::Reg(A), Src::Reg(H)), 1, 1),
            0x7D => n(LD8(Src::Reg(A), Src::Reg(L)), 1, 1),
            0x7E => n(LD8(Src::Reg(A), Src::Deref(HL)), 2, 1),
            0x7F => n(LD8(Src::Reg(A), Src::Reg(A)), 1, 1),
            0x80 => n(ADD8(Src::Reg(B)), 1, 1),
            0x81 => n(ADD8(Src::Reg(C)), 1, 1),
            0x82 => n(ADD8(Src::Reg(D)), 1, 1),
            0x83 => n(ADD8(Src::Reg(E)), 1, 1),
            0x84 => n(ADD8(Src::Reg(H)), 1, 1),
            0x85 => n(ADD8(Src::Reg(L)), 1, 1),
            0x86 => n(ADD8(Src::Deref(HL)), 2, 1),
            0x87 => n(ADD8(Src::Reg(A)), 1, 1),
            0x88 => n(ADC8(Src::Reg(B)), 1, 1),
            0x89 => n(ADC8(Src::Reg(C)), 1, 1),
            0x8A => n(ADC8(Src::Reg(D)), 1, 1),
            0x8B => n(ADC8(Src::Reg(E)), 1, 1),
            0x8C => n(ADC8(Src::Reg(H)), 1, 1),
            0x8D => n(ADC8(Src::Reg(L)), 1, 1),
            0x8E => n(ADC8(Src::Deref(HL)), 2, 1),
            0x8F => n(ADC8(Src::Reg(A)), 1, 1),
            0x90 => n(SUB8(Src::Reg(B)), 1, 1),
            0x91 => n(SUB8(Src::Reg(C)), 1, 1),
            0x92 => n(SUB8(Src::Reg(D)), 1, 1),
            0x93 => n(SUB8(Src::Reg(E)), 1, 1),
            0x94 => n(SUB8(Src::Reg(H)), 1, 1),
            0x95 => n(SUB8(Src::Reg(L)), 1, 1),
            0x96 => n(SUB8(Src::Deref(HL)), 2, 1),
            0x97 => n(SUB8(Src::Reg(A)), 1, 1),
            0x98 => n(SBC8(Src::Reg(B)), 1, 1),
            0x99 => n(SBC8(Src::Reg(C)), 1, 1),
            0x9A => n(SBC8(Src::Reg(D)), 1, 1),
            0x9B => n(SBC8(Src::Reg(E)), 1, 1),
            0x9C => n(SBC8(Src::Reg(H)), 1, 1),
            0x9D => n(SBC8(Src::Reg(L)), 1, 1),
            0x9E => n(SBC8(Src::Deref(HL)), 2, 1),
            0x9F => n(SBC8(Src::Reg(A)), 1, 1),
            0xA0 => n(AND(Src::Reg(B)), 1, 1),
            0xA1 => n(AND(Src::Reg(C)), 1, 1),
            0xA2 => n(AND(Src::Reg(D)), 1, 1),
            0xA3 => n(AND(Src::Reg(E)), 1, 1),
            0xA4 => n(AND(Src::Reg(H)), 1, 1),
            0xA5 => n(AND(Src::Reg(L)), 1, 1),
            0xA6 => n(AND(Src::Deref(HL)), 2, 1),
            0xA7 => n(AND(Src::Reg(A)), 1, 1),
            0xA8 => n(XOR(Src::Reg(B)), 1, 1),
            0xA9 => n(XOR(Src::Reg(C)), 1, 1),
            0xAA => n(XOR(Src::Reg(D)), 1, 1),
            0xAB => n(XOR(Src::Reg(E)), 1, 1),
            0xAC => n(XOR(Src::Reg(H)), 1, 1),
            0xAD => n(XOR(Src::Reg(L)), 1, 1),
            0xAE => n(XOR(Src::Deref(HL)), 2, 1),
            0xAF => n(XOR(Src::Reg(A)), 1, 1),
            0xB0 => n(OR(Src::Reg(B)), 1, 1),
            0xB1 => n(OR(Src::Reg(C)), 1, 1),
            0xB2 => n(OR(Src::Reg(D)), 1, 1),
            0xB3 => n(OR(Src::Reg(E)), 1, 1),
            0xB4 => n(OR(Src::Reg(H)), 1, 1),
            0xB5 => n(OR(Src::Reg(L)), 1, 1),
            0xB6 => n(OR(Src::Deref(HL)), 2, 1),
            0xB7 => n(OR(Src::Reg(A)), 1, 1),
            0xB8 => n(CP(Src::Reg(B)), 1, 1),
            0xB9 => n(CP(Src::Reg(C)), 1, 1),
            0xBA => n(CP(Src::Reg(D)), 1, 1),
            0xBB => n(CP(Src::Reg(E)), 1, 1),
            0xBC => n(CP(Src::Reg(H)), 1, 1),
            0xBD => n(CP(Src::Reg(L)), 1, 1),
            0xBE => n(CP(Src::Deref(HL)), 2, 1),
            0xBF => n(CP(Src::Reg(A)), 1, 1),
            0xC0 => n(RET(FlagCondition::NZ), 5, 1),
            0xC1 => n(POP(BC), 4, 1),
            0xC2 => n(JP(FlagCondition::NZ, Src::D16(b21)), 4, 3),
            0xC3 => n(JP(FlagCondition::ALWAYS, Src::D16(b21)), 4, 3),
            0xC4 => n(CALL(FlagCondition::NZ, b21), 6, 3),
            0xC5 => n(PUSH(BC), 4, 1),
            0xC6 => n(ADD8(Src::D8(b1)), 2, 2),
            0xC7 => n(RST(0x00), 4, 1),
            0xC8 => n(RET(FlagCondition::Z), 5, 1),
            0xC9 => n(RET(FlagCondition::ALWAYS), 4, 1),
            0xCA => n(JP(FlagCondition::Z, Src::D16(b21)), 4, 3),
            0xCC => n(CALL(FlagCondition::Z, b21), 6, 3),
            0xCD => n(CALL(FlagCondition::ALWAYS, b21), 6, 3),
            0xCE => n(ADC8(Src::D8(b1)), 2, 2),
            0xCF => n(RST(0x08), 4, 1),
            0xD0 => n(RET(FlagCondition::NC), 5, 1),
            0xD1 => n(POP(DE), 4, 1),
            0xD2 => n(JP(FlagCondition::NC, Src::D16(b21)), 4, 3),
            0xD3 => n(INVALID(b0), 1, 1),
            0xD4 => n(CALL(FlagCondition::NC, b21), 6, 3),
            0xD5 => n(PUSH(DE), 4, 1),
            0xD6 => n(SUB8(Src::D8(b1)), 2, 2),
            0xD7 => n(RST(0x10), 4, 1),
            0xD8 => n(RET(FlagCondition::C), 5, 1),
            0xD9 => n(RETI, 4, 1),
            0xDA => n(JP(FlagCondition::C, Src::D16(b21)), 4, 3),
            0xDB => n(INVALID(b0), 1, 1),
            0xDC => n(CALL(FlagCondition::C, b21), 6, 3),
            0xDD => n(INVALID(b0), 1, 1),
            0xDE => n(SBC8(Src::D8(b1)), 2, 2),
            0xDF => n(RST(0x18), 4, 1),
            0xE0 => n(LD8(Src::A8(b1), Src::Reg(A)), 3, 2),
            0xE1 => n(POP(HL), 4, 1),
            0xE2 => n(LD8(Src::Deref(C), Src::Reg(A)), 2, 1),
            0xE3 => n(INVALID(b0), 1, 1),
            0xE4 => n(INVALID(b0), 1, 1),
            0xE5 => n(PUSH(HL), 4, 1),
            0xE6 => n(AND(Src::D8(b1)), 2, 2),
            0xE7 => n(RST(0x20), 4, 1),
            0xE8 => n(ADD16(SP, Src::I8(b1 as i8)), 4, 2),
            0xE9 => n(JP(FlagCondition::ALWAYS, Src::Reg(HL)), 1, 1),
            0xEA => n(LD8(Src::A16(b21), Src::Reg(A)), 4, 3),
            0xEB => n(INVALID(b0), 1, 1),
            0xEC => n(INVALID(b0), 1, 1),
            0xED => n(INVALID(b0), 1, 1),
            0xEE => n(XOR(Src::D8(b1)), 2, 2),
            0xEF => n(RST(0x28), 4, 1),
            0xF0 => n(LD8(Src::Reg(A), Src::A8(b1)), 3, 2),
            0xF1 => n(POP(AF), 4, 1),
            0xF2 => n(LD8(Src::Reg(A), Src::Deref(C)), 2, 1),
            0xF3 => n(DI, 1, 1),
            0xF4 => n(INVALID(b0), 1, 1),
            0xF5 => n(PUSH(AF), 4, 1),
            0xF6 => n(OR(Src::D8(b1)), 2, 2),
            0xF7 => n(RST(0x30), 4, 1),
            0xF8 => n(LD16(Src::Reg(HL), Src::I8(b1 as i8)), 3, 2),
            0xF9 => n(LD16(Src::Reg(SP), Src::Reg(HL)), 2, 1),
            0xFA => n(LD8(Src::Reg(A), Src::A16(b21)), 4, 3),
            0xFB => n(EI, 1, 1),
            0xFC => n(INVALID(b0), 1, 1),
            0xFD => n(INVALID(b0), 1, 1),
            0xFE => n(CP(Src::D8(b1)), 2, 2),
            0xFF => n(RST(0x38), 4, 1),
            0xCB => match b1 {
                0x00 => n(RLC(Src::Reg(B)), 2, 2),
                0x01 => n(RLC(Src::Reg(C)), 2, 2),
                0x02 => n(RLC(Src::Reg(D)), 2, 2),
                0x03 => n(RLC(Src::Reg(E)), 2, 2),
                0x04 => n(RLC(Src::Reg(H)), 2, 2),
                0x05 => n(RLC(Src::Reg(L)), 2, 2),
                0x06 => n(RLC(Src::Deref(HL)), 4, 2),
                0x07 => n(RLC(Src::Reg(A)), 2, 2),
                0x08 => n(RRC(Src::Reg(B)), 2, 2),
                0x09 => n(RRC(Src::Reg(C)), 2, 2),
                0x0A => n(RRC(Src::Reg(D)), 2, 2),
                0x0B => n(RRC(Src::Reg(E)), 2, 2),
                0x0C => n(RRC(Src::Reg(H)), 2, 2),
                0x0D => n(RRC(Src::Reg(L)), 2, 2),
                0x0E => n(RRC(Src::Deref(HL)), 4, 2),
                0x0F => n(RRC(Src::Reg(A)), 2, 2),
                0x10 => n(RL(Src::Reg(B)), 2, 2),
                0x11 => n(RL(Src::Reg(C)), 2, 2),
                0x12 => n(RL(Src::Reg(D)), 2, 2),
                0x13 => n(RL(Src::Reg(E)), 2, 2),
                0x14 => n(RL(Src::Reg(H)), 2, 2),
                0x15 => n(RL(Src::Reg(L)), 2, 2),
                0x16 => n(RL(Src::Deref(HL)), 4, 2),
                0x17 => n(RL(Src::Reg(A)), 2, 2),
                0x18 => n(RR(Src::Reg(B)), 2, 2),
                0x19 => n(RR(Src::Reg(C)), 2, 2),
                0x1A => n(RR(Src::Reg(D)), 2, 2),
                0x1B => n(RR(Src::Reg(E)), 2, 2),
                0x1C => n(RR(Src::Reg(H)), 2, 2),
                0x1D => n(RR(Src::Reg(L)), 2, 2),
                0x1E => n(RR(Src::Deref(HL)), 4, 2),
                0x1F => n(RR(Src::Reg(A)), 2, 2),
                0x20 => n(SLA(Src::Reg(B)), 2, 2),
                0x21 => n(SLA(Src::Reg(C)), 2, 2),
                0x22 => n(SLA(Src::Reg(D)), 2, 2),
                0x23 => n(SLA(Src::Reg(E)), 2, 2),
                0x24 => n(SLA(Src::Reg(H)), 2, 2),
                0x25 => n(SLA(Src::Reg(L)), 2, 2),
                0x26 => n(SLA(Src::Deref(HL)), 4, 2),
                0x27 => n(SLA(Src::Reg(A)), 2, 2),
                0x28 => n(SRA(Src::Reg(B)), 2, 2),
                0x29 => n(SRA(Src::Reg(C)), 2, 2),
                0x2A => n(SRA(Src::Reg(D)), 2, 2),
                0x2B => n(SRA(Src::Reg(E)), 2, 2),
                0x2C => n(SRA(Src::Reg(H)), 2, 2),
                0x2D => n(SRA(Src::Reg(L)), 2, 2),
                0x2E => n(SRA(Src::Deref(HL)), 4, 2),
                0x2F => n(SRA(Src::Reg(A)), 2, 2),
                0x30 => n(SWAP(Src::Reg(B)), 2, 2),
                0x31 => n(SWAP(Src::Reg(C)), 2, 2),
                0x32 => n(SWAP(Src::Reg(D)), 2, 2),
                0x33 => n(SWAP(Src::Reg(E)), 2, 2),
                0x34 => n(SWAP(Src::Reg(H)), 2, 2),
                0x35 => n(SWAP(Src::Reg(L)), 2, 2),
                0x36 => n(SWAP(Src::Deref(HL)), 4, 2),
                0x37 => n(SWAP(Src::Reg(A)), 2, 2),
                0x38 => n(SRL(Src::Reg(B)), 2, 2),
                0x39 => n(SRL(Src::Reg(C)), 2, 2),
                0x3A => n(SRL(Src::Reg(D)), 2, 2),
                0x3B => n(SRL(Src::Reg(E)), 2, 2),
                0x3C => n(SRL(Src::Reg(H)), 2, 2),
                0x3D => n(SRL(Src::Reg(L)), 2, 2),
                0x3E => n(SRL(Src::Deref(HL)), 4, 2),
                0x3F => n(SRL(Src::Reg(A)), 2, 2),
                0x40 => n(BIT(0, Src::Reg(B)), 2, 2),
                0x41 => n(BIT(0, Src::Reg(C)), 2, 2),
                0x42 => n(BIT(0, Src::Reg(D)), 2, 2),
                0x43 => n(BIT(0, Src::Reg(E)), 2, 2),
                0x44 => n(BIT(0, Src::Reg(H)), 2, 2),
                0x45 => n(BIT(0, Src::Reg(L)), 2, 2),
                0x46 => n(BIT(0, Src::Deref(HL)), 3, 2),
                0x47 => n(BIT(0, Src::Reg(A)), 2, 2),
                0x48 => n(BIT(1, Src::Reg(B)), 2, 2),
                0x49 => n(BIT(1, Src::Reg(C)), 2, 2),
                0x4A => n(BIT(1, Src::Reg(D)), 2, 2),
                0x4B => n(BIT(1, Src::Reg(E)), 2, 2),
                0x4C => n(BIT(1, Src::Reg(H)), 2, 2),
                0x4D => n(BIT(1, Src::Reg(L)), 2, 2),
                0x4E => n(BIT(1, Src::Deref(HL)), 3, 2),
                0x4F => n(BIT(1, Src::Reg(A)), 2, 2),
                0x50 => n(BIT(2, Src::Reg(B)), 2, 2),
                0x51 => n(BIT(2, Src::Reg(C)), 2, 2),
                0x52 => n(BIT(2, Src::Reg(D)), 2, 2),
                0x53 => n(BIT(2, Src::Reg(E)), 2, 2),
                0x54 => n(BIT(2, Src::Reg(H)), 2, 2),
                0x55 => n(BIT(2, Src::Reg(L)), 2, 2),
                0x56 => n(BIT(2, Src::Deref(HL)), 3, 2),
                0x57 => n(BIT(2, Src::Reg(A)), 2, 2),
                0x58 => n(BIT(3, Src::Reg(B)), 2, 2),
                0x59 => n(BIT(3, Src::Reg(C)), 2, 2),
                0x5A => n(BIT(3, Src::Reg(D)), 2, 2),
                0x5B => n(BIT(3, Src::Reg(E)), 2, 2),
                0x5C => n(BIT(3, Src::Reg(H)), 2, 2),
                0x5D => n(BIT(3, Src::Reg(L)), 2, 2),
                0x5E => n(BIT(3, Src::Deref(HL)), 3, 2),
                0x5F => n(BIT(3, Src::Reg(A)), 2, 2),
                0x60 => n(BIT(4, Src::Reg(B)), 2, 2),
                0x61 => n(BIT(4, Src::Reg(C)), 2, 2),
                0x62 => n(BIT(4, Src::Reg(D)), 2, 2),
                0x63 => n(BIT(4, Src::Reg(E)), 2, 2),
                0x64 => n(BIT(4, Src::Reg(H)), 2, 2),
                0x65 => n(BIT(4, Src::Reg(L)), 2, 2),
                0x66 => n(BIT(4, Src::Deref(HL)), 3, 2),
                0x67 => n(BIT(4, Src::Reg(A)), 2, 2),
                0x68 => n(BIT(5, Src::Reg(B)), 2, 2),
                0x69 => n(BIT(5, Src::Reg(C)), 2, 2),
                0x6A => n(BIT(5, Src::Reg(D)), 2, 2),
                0x6B => n(BIT(5, Src::Reg(E)), 2, 2),
                0x6C => n(BIT(5, Src::Reg(H)), 2, 2),
                0x6D => n(BIT(5, Src::Reg(L)), 2, 2),
                0x6E => n(BIT(5, Src::Deref(HL)), 3, 2),
                0x6F => n(BIT(5, Src::Reg(A)), 2, 2),
                0x70 => n(BIT(6, Src::Reg(B)), 2, 2),
                0x71 => n(BIT(6, Src::Reg(C)), 2, 2),
                0x72 => n(BIT(6, Src::Reg(D)), 2, 2),
                0x73 => n(BIT(6, Src::Reg(E)), 2, 2),
                0x74 => n(BIT(6, Src::Reg(H)), 2, 2),
                0x75 => n(BIT(6, Src::Reg(L)), 2, 2),
                0x76 => n(BIT(6, Src::Deref(HL)), 3, 2),
                0x77 => n(BIT(6, Src::Reg(A)), 2, 2),
                0x78 => n(BIT(7, Src::Reg(B)), 2, 2),
                0x79 => n(BIT(7, Src::Reg(C)), 2, 2),
                0x7A => n(BIT(7, Src::Reg(D)), 2, 2),
                0x7B => n(BIT(7, Src::Reg(E)), 2, 2),
                0x7C => n(BIT(7, Src::Reg(H)), 2, 2),
                0x7D => n(BIT(7, Src::Reg(L)), 2, 2),
                0x7E => n(BIT(7, Src::Deref(HL)), 3, 2),
                0x7F => n(BIT(7, Src::Reg(A)), 2, 2),
                0x80 => n(RES(0, Src::Reg(B)), 2, 2),
                0x81 => n(RES(0, Src::Reg(C)), 2, 2),
                0x82 => n(RES(0, Src::Reg(D)), 2, 2),
                0x83 => n(RES(0, Src::Reg(E)), 2, 2),
                0x84 => n(RES(0, Src::Reg(H)), 2, 2),
                0x85 => n(RES(0, Src::Reg(L)), 2, 2),
                0x86 => n(RES(0, Src::Deref(HL)), 4, 2),
                0x87 => n(RES(0, Src::Reg(A)), 2, 2),
                0x88 => n(RES(1, Src::Reg(B)), 2, 2),
                0x89 => n(RES(1, Src::Reg(C)), 2, 2),
                0x8A => n(RES(1, Src::Reg(D)), 2, 2),
                0x8B => n(RES(1, Src::Reg(E)), 2, 2),
                0x8C => n(RES(1, Src::Reg(H)), 2, 2),
                0x8D => n(RES(1, Src::Reg(L)), 2, 2),
                0x8E => n(RES(1, Src::Deref(HL)), 4, 2),
                0x8F => n(RES(1, Src::Reg(A)), 2, 2),
                0x90 => n(RES(2, Src::Reg(B)), 2, 2),
                0x91 => n(RES(2, Src::Reg(C)), 2, 2),
                0x92 => n(RES(2, Src::Reg(D)), 2, 2),
                0x93 => n(RES(2, Src::Reg(E)), 2, 2),
                0x94 => n(RES(2, Src::Reg(H)), 2, 2),
                0x95 => n(RES(2, Src::Reg(L)), 2, 2),
                0x96 => n(RES(2, Src::Deref(HL)), 4, 2),
                0x97 => n(RES(2, Src::Reg(A)), 2, 2),
                0x98 => n(RES(3, Src::Reg(B)), 2, 2),
                0x99 => n(RES(3, Src::Reg(C)), 2, 2),
                0x9A => n(RES(3, Src::Reg(D)), 2, 2),
                0x9B => n(RES(3, Src::Reg(E)), 2, 2),
                0x9C => n(RES(3, Src::Reg(H)), 2, 2),
                0x9D => n(RES(3, Src::Reg(L)), 2, 2),
                0x9E => n(RES(3, Src::Deref(HL)), 4, 2),
                0x9F => n(RES(3, Src::Reg(A)), 2, 2),
                0xA0 => n(RES(4, Src::Reg(B)), 2, 2),
                0xA1 => n(RES(4, Src::Reg(C)), 2, 2),
                0xA2 => n(RES(4, Src::Reg(D)), 2, 2),
                0xA3 => n(RES(4, Src::Reg(E)), 2, 2),
                0xA4 => n(RES(4, Src::Reg(H)), 2, 2),
                0xA5 => n(RES(4, Src::Reg(L)), 2, 2),
                0xA6 => n(RES(4, Src::Deref(HL)), 4, 2),
                0xA7 => n(RES(4, Src::Reg(A)), 2, 2),
                0xA8 => n(RES(5, Src::Reg(B)), 2, 2),
                0xA9 => n(RES(5, Src::Reg(C)), 2, 2),
                0xAA => n(RES(5, Src::Reg(D)), 2, 2),
                0xAB => n(RES(5, Src::Reg(E)), 2, 2),
                0xAC => n(RES(5, Src::Reg(H)), 2, 2),
                0xAD => n(RES(5, Src::Reg(L)), 2, 2),
                0xAE => n(RES(5, Src::Deref(HL)), 4, 2),
                0xAF => n(RES(5, Src::Reg(A)), 2, 2),
                0xB0 => n(RES(6, Src::Reg(B)), 2, 2),
                0xB1 => n(RES(6, Src::Reg(C)), 2, 2),
                0xB2 => n(RES(6, Src::Reg(D)), 2, 2),
                0xB3 => n(RES(6, Src::Reg(E)), 2, 2),
                0xB4 => n(RES(6, Src::Reg(H)), 2, 2),
                0xB5 => n(RES(6, Src::Reg(L)), 2, 2),
                0xB6 => n(RES(6, Src::Deref(HL)), 4, 2),
                0xB7 => n(RES(6, Src::Reg(A)), 2, 2),
                0xB8 => n(RES(7, Src::Reg(B)), 2, 2),
                0xB9 => n(RES(7, Src::Reg(C)), 2, 2),
                0xBA => n(RES(7, Src::Reg(D)), 2, 2),
                0xBB => n(RES(7, Src::Reg(E)), 2, 2),
                0xBC => n(RES(7, Src::Reg(H)), 2, 2),
                0xBD => n(RES(7, Src::Reg(L)), 2, 2),
                0xBE => n(RES(7, Src::Deref(HL)), 4, 2),
                0xBF => n(RES(7, Src::Reg(A)), 2, 2),
                0xC0 => n(SET(0, Src::Reg(B)), 2, 2),
                0xC1 => n(SET(0, Src::Reg(C)), 2, 2),
                0xC2 => n(SET(0, Src::Reg(D)), 2, 2),
                0xC3 => n(SET(0, Src::Reg(E)), 2, 2),
                0xC4 => n(SET(0, Src::Reg(H)), 2, 2),
                0xC5 => n(SET(0, Src::Reg(L)), 2, 2),
                0xC6 => n(SET(0, Src::Deref(HL)), 4, 2),
                0xC7 => n(SET(0, Src::Reg(A)), 2, 2),
                0xC8 => n(SET(1, Src::Reg(B)), 2, 2),
                0xC9 => n(SET(1, Src::Reg(C)), 2, 2),
                0xCA => n(SET(1, Src::Reg(D)), 2, 2),
                0xCB => n(SET(1, Src::Reg(E)), 2, 2),
                0xCC => n(SET(1, Src::Reg(H)), 2, 2),
                0xCD => n(SET(1, Src::Reg(L)), 2, 2),
                0xCE => n(SET(1, Src::Deref(HL)), 4, 2),
                0xCF => n(SET(1, Src::Reg(A)), 2, 2),
                0xD0 => n(SET(2, Src::Reg(B)), 2, 2),
                0xD1 => n(SET(2, Src::Reg(C)), 2, 2),
                0xD2 => n(SET(2, Src::Reg(D)), 2, 2),
                0xD3 => n(SET(2, Src::Reg(E)), 2, 2),
                0xD4 => n(SET(2, Src::Reg(H)), 2, 2),
                0xD5 => n(SET(2, Src::Reg(L)), 2, 2),
                0xD6 => n(SET(2, Src::Deref(HL)), 4, 2),
                0xD7 => n(SET(2, Src::Reg(A)), 2, 2),
                0xD8 => n(SET(3, Src::Reg(B)), 2, 2),
                0xD9 => n(SET(3, Src::Reg(C)), 2, 2),
                0xDA => n(SET(3, Src::Reg(D)), 2, 2),
                0xDB => n(SET(3, Src::Reg(E)), 2, 2),
                0xDC => n(SET(3, Src::Reg(H)), 2, 2),
                0xDD => n(SET(3, Src::Reg(L)), 2, 2),
                0xDE => n(SET(3, Src::Deref(HL)), 4, 2),
                0xDF => n(SET(3, Src::Reg(A)), 2, 2),
                0xE0 => n(SET(4, Src::Reg(B)), 2, 2),
                0xE1 => n(SET(4, Src::Reg(C)), 2, 2),
                0xE2 => n(SET(4, Src::Reg(D)), 2, 2),
                0xE3 => n(SET(4, Src::Reg(E)), 2, 2),
                0xE4 => n(SET(4, Src::Reg(H)), 2, 2),
                0xE5 => n(SET(4, Src::Reg(L)), 2, 2),
                0xE6 => n(SET(4, Src::Deref(HL)), 4, 2),
                0xE7 => n(SET(4, Src::Reg(A)), 2, 2),
                0xE8 => n(SET(5, Src::Reg(B)), 2, 2),
                0xE9 => n(SET(5, Src::Reg(C)), 2, 2),
                0xEA => n(SET(5, Src::Reg(D)), 2, 2),
                0xEB => n(SET(5, Src::Reg(E)), 2, 2),
                0xEC => n(SET(5, Src::Reg(H)), 2, 2),
                0xED => n(SET(5, Src::Reg(L)), 2, 2),
                0xEE => n(SET(5, Src::Deref(HL)), 4, 2),
                0xEF => n(SET(5, Src::Reg(A)), 2, 2),
                0xF0 => n(SET(6, Src::Reg(B)), 2, 2),
                0xF1 => n(SET(6, Src::Reg(C)), 2, 2),
                0xF2 => n(SET(6, Src::Reg(D)), 2, 2),
                0xF3 => n(SET(6, Src::Reg(E)), 2, 2),
                0xF4 => n(SET(6, Src::Reg(H)), 2, 2),
                0xF5 => n(SET(6, Src::Reg(L)), 2, 2),
                0xF6 => n(SET(6, Src::Deref(HL)), 4, 2),
                0xF7 => n(SET(6, Src::Reg(A)), 2, 2),
                0xF8 => n(SET(7, Src::Reg(B)), 2, 2),
                0xF9 => n(SET(7, Src::Reg(C)), 2, 2),
                0xFA => n(SET(7, Src::Reg(D)), 2, 2),
                0xFB => n(SET(7, Src::Reg(E)), 2, 2),
                0xFC => n(SET(7, Src::Reg(H)), 2, 2),
                0xFD => n(SET(7, Src::Reg(L)), 2, 2),
                0xFE => n(SET(7, Src::Deref(HL)), 4, 2),
                0xFF => n(SET(7, Src::Reg(A)), 2, 2)
            }
        }
    }
}
