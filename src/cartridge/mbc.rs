#[derive(Debug)]
pub enum MbcType {
    RomOnly,
    Mbc1,
    Mbc3,
    Mbc5
}

enum Mode {
    Rom,
    Ram
}

pub trait Mbc {
    fn rom_bank_num(&self) -> usize;
    fn get8(&self, addr: u16) -> u8;
    fn set8(&mut self, addr: u16, byte: u8);
    fn mbc_type(&self) -> MbcType;
    fn dump_ram(&self) -> Vec<u8>;
    fn load_ram(&mut self, data: &[u8]);
}

struct RomOnly {
    ram_bank: [u8; 0x2000]
}

impl RomOnly {
    fn new() -> Self {
        RomOnly {
            ram_bank: [0; 0x2000]
        }
    }
}

impl Mbc for RomOnly {
    fn rom_bank_num(&self) -> usize {
        1
    }

    fn set8(&mut self, addr: u16, _byte: u8) {
        debug!("set8 {:x} in RomOnly", addr);
    }

    fn get8(&self, addr: u16) -> u8 {
        debug!("get8 {:x} in RomOnly", addr);
        self.ram_bank[addr as usize - 0xA000]
    }

    fn mbc_type(&self) -> MbcType {
        MbcType::RomOnly
    }

    fn dump_ram(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_ram(&mut self, data: &[u8]) { }
}

struct Mbc1 {
    five_bit_reg: u8,
    two_bit_reg: u8,
    ram_bank_enabled: bool,
    mode: Mode,
    ram: Vec<u8>
}

impl Mbc1 {
    fn new() -> Self {
        Mbc1 {
            five_bit_reg: 0,
            two_bit_reg: 0,
            ram_bank_enabled: false,
            mode: Mode::Rom,
            ram: vec![0; 0x8000]
        }
    }

    fn ram_bank_offset(&self) -> usize {
        match self.mode {
            Mode::Rom => 0,
            Mode::Ram => (self.two_bit_reg as usize) * 0x2000
        }
    }
}

impl Mbc for Mbc1 {
    fn rom_bank_num(&self) -> usize {
        match self.mode {
            Mode::Rom => {
                let n = ((self.two_bit_reg << 5) | self.five_bit_reg) as usize;
                match n {
                    0x00 | 0x20 | 0x40 | 0x60 => n + 1,
                    _ => n
                }
            }
            Mode::Ram => 1
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        self.ram[self.ram_bank_offset() + (addr as usize) - 0xA000]
    }

    fn set8(&mut self, addr: u16, byte: u8) {
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
            0xA000 ... 0xBFFF => {
                let offset = self.ram_bank_offset();
                self.ram[offset + (addr as usize) - 0xA000] = byte;
            }
            _ => panic!("Can't write to MBC1 at 0x{:X}: 0x{:X}", addr, byte)
        }
    }

    fn mbc_type(&self) -> MbcType {
        MbcType::Mbc1
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, data: &[u8]) {
        self.ram = data.to_owned();
    }
}

struct Mbc3 {
    ram_rtc_bank_enabled: bool,
    rom_bank_reg: u8,
    ram_rtc_bank_reg: u8,
    ram: Vec<u8>
}

impl Mbc3 {
    fn new() -> Self {
        Mbc3 {
            ram_rtc_bank_enabled: false,
            rom_bank_reg: 0,
            ram_rtc_bank_reg: 0,
            ram: vec![0; 0x8000]
        }
    }
}

impl Mbc for Mbc3 {
    fn rom_bank_num(&self) -> usize {
        match self.rom_bank_reg {
            0x00 => 1,
            n => n as usize
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        match addr {
            0xA000 ... 0xBFFF => {
                match self.ram_rtc_bank_reg {
                    0x0 ... 0x3 => {
                        let bank_offset = (self.ram_rtc_bank_reg as usize) * 0x2000;
                        self.ram[bank_offset + (addr as usize) - 0xA000]
                    }
                    0x8 ... 0xC => {
//                        debug!("GET RTC REGISTER");
                        0
                    }
                    _ => panic!("Invalid ram_rtc_bank_reg {:X}", self.ram_rtc_bank_reg)
                }
            }
            _ => panic!("Can't get MBC3 at 0x{:X}", addr)
        }
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000 ... 0x1FFF => {
                self.ram_rtc_bank_enabled = byte & 0x0A == 0x0A;
            }
            0x2000 ... 0x3FFF => {
                self.rom_bank_reg = byte & 0x7F;
            }
            0x4000 ... 0x5FFF => {
                self.ram_rtc_bank_reg = byte;
            }
            0x6000 ... 0x7FFF => {
//                debug!("LATCH");
            }
            0xA000 ... 0xBFFF => {
                match self.ram_rtc_bank_reg {
                    0x0 ... 0x3 => {
                        let bank_offset = (self.ram_rtc_bank_reg as usize) * 0x2000;
                        self.ram[bank_offset + (addr as usize) - 0xA000] = byte;
                    }
                    0x8 ... 0xC => {
//                        debug!("SET RTC REGISTER");
                    }
                    _ => panic!("Invalid ram_rtc_bank_reg {:X}", self.ram_rtc_bank_reg)
                }
            }
            _ => panic!("Can't write to MBC3 at 0x{:X}: 0x{:X}", addr, byte)
        }
    }

    fn mbc_type(&self) -> MbcType {
        MbcType::Mbc3
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, data: &[u8]) {
        self.ram = data.to_owned();
    }
}

struct Mbc5 {
    rom_bank_num: u16,
    ram_bank_num: u8,
    ram_bank_enabled: bool,
    mode: Mode,
    ram: Vec<u8>
}

impl Mbc5 {
    fn new() -> Self {
        Mbc5 {
            rom_bank_num: 0,
            ram_bank_num: 0,
            ram_bank_enabled: false,
            mode: Mode::Rom,
            ram: vec![0xFF; 0x8000]
        }
    }

    fn ram_bank_offset(&self) -> usize {
        match self.mode {
            Mode::Rom => 0,
            Mode::Ram => (self.ram_bank_num as usize) * 0x2000
        }
    }
}

impl Mbc for Mbc5 {
    fn rom_bank_num(&self) -> usize {
        match self.mode {
            Mode::Rom => self.rom_bank_num as usize,
            Mode::Ram => 1
        }
    }

    fn get8(&self, addr: u16) -> u8 {
        self.ram[self.ram_bank_offset() + (addr as usize) - 0xA000]
    }

    fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            0x0000 ... 0x1FFF => {
                self.ram_bank_enabled = byte & 0x0A == 0x0A;
            }
            0x2000 ... 0x2FFF => {
                self.rom_bank_num = (self.rom_bank_num & 0x100) | (byte as u16);
            }
            0x3000 ... 0x3FFF => {
                self.rom_bank_num = ((byte as u16 & 0x1) << 8) | (self.rom_bank_num & 0xFF);
            }
            0x4000 ... 0x5FFF => {
                self.ram_bank_num = byte & 0x03;
            }
            0x6000 ... 0x7FFF => {
                self.mode = if byte & 0x1  == 0 {
                    Mode::Rom
                } else {
                    Mode::Ram
                }
            }
            0xA000 ... 0xBFFF => {
                let offset = self.ram_bank_offset();
                self.ram[offset + (addr as usize) - 0xA000] = byte;
            }
            _ => panic!("Can't write to MBC1 at 0x{:X}: 0x{:X}", addr, byte)
        }
    }

    fn mbc_type(&self) -> MbcType {
        MbcType::Mbc5
    }

    fn dump_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, data: &[u8]) {
        self.ram = data.to_owned();
        debug!("LOADED: {:X}", self.ram.len());
    }
}

pub fn build_mbc(byte: u8) -> Box<Mbc> {
    match byte {
        0 => Box::new(RomOnly::new()),
        1...3 => Box::new(Mbc1::new()),
        5...6 => Box::new(Mbc1::new()),
        0xF...0x13 => Box::new(Mbc3::new()),
        0x19...0x1E => Box::new(Mbc5::new()),
        _ => panic!("Unsupported mbc type: {:X}", byte)
    }
}
