#[derive(Debug)]
pub enum MbcType {
    RomOnly = 0,
    MBC1 = 1
}

enum Mode {
    Rom,
    Ram
}

pub trait Mbc {
    fn rom_bank_num(&self) -> u8;
    fn ram_bank_num(&self) -> u8;
    fn set8(&mut self, addr: u16, byte: u8);
    fn mbc_type(&self) -> MbcType;
}

struct RomOnly { }

impl RomOnly {
    fn new() -> Self {
        RomOnly { }
    }
}

impl Mbc for RomOnly {
    fn rom_bank_num(&self) -> u8 {
        1
    }

    fn ram_bank_num(&self) -> u8 {
        0
    }

    fn set8(&mut self, _addr: u16, _byte: u8) { }

    fn mbc_type(&self) -> MbcType {
        MbcType::RomOnly
    }
}

struct Mbc1 {
    five_bit_reg: u8,
    two_bit_reg: u8,
    ram_bank_enabled: bool,
    mode: Mode
}

impl Mbc1 {
    fn new() -> Self {
        Mbc1 {
            five_bit_reg: 0,
            two_bit_reg: 0,
            ram_bank_enabled: false,
            mode: Mode::Rom
        }
    }
}

impl Mbc for Mbc1 {
    fn rom_bank_num(&self) -> u8 {
        match self.mode {
            Mode::Rom => {
                let n = (self.two_bit_reg << 6) | self.five_bit_reg;
                match n {
                    0x00 | 0x20 | 0x40 | 0x60 => n + 1,
                    _ => n
                }
            }
            Mode::Ram => 1
        }
    }

    fn ram_bank_num(&self) -> u8 {
        match self.mode {
            Mode::Rom => 0,
            Mode::Ram => self.two_bit_reg
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        debug!("MBC1 SET: {:X}, {:X}", addr, byte);
        match addr {
            0x0000 ... 0x1FFF => {
                self.ram_bank_enabled = byte & 0x0A == 0x0A;
            }
            0x2000 ... 0x3FFF => {
                self.five_bit_reg = byte & 0x1F;
            }
            0x4000 ... 0x5FFF => {
                self.two_bit_reg = byte & 0x03;
            }
            0x6000 ... 0x7FFF => {
                self.mode = if byte & 0x1  == 0 {
                    Mode::Rom
                } else {
                    Mode::Ram
                }
            }
            _ => panic!("Can't write to MBC1 at 0x{:X}: 0x{:X}", addr, byte)
        }
    }

    fn mbc_type(&self) -> MbcType {
        MbcType::MBC1
    }
}

pub fn build_mbc(byte: u8) -> Box<Mbc> {
    match byte {
        0 => Box::new(RomOnly::new()),
        1 => Box::new(Mbc1::new()),
        _ => panic!("Unsupported mbc type: {}", byte)
    }
}
