use super::memory_map::{MemoryMap, MemoryMappedDeviceManager, MemoryMappedDevice};

const DMA: u16 = 0xFF46;
const HDMA5: u16 = 0xFF55;

pub struct MemoryBus<'a> {
    memory_map: &'a MemoryMap,
    devices: &'a mut MemoryMappedDeviceManager
}

impl<'a> MemoryBus<'a> {
    pub fn new(memory_map: &'a MemoryMap, devices: &'a mut MemoryMappedDeviceManager) -> MemoryBus<'a> {
        MemoryBus { memory_map, devices }
    }

    pub fn devices(&mut self) -> &mut MemoryMappedDeviceManager {
        self.devices
    }

    fn get_device(&mut self, addr: u16) -> &mut MemoryMappedDevice {
        self.devices.get(self.memory_map.get_id(addr))
    }

    pub fn get_sym(&self, addr: u16) -> Option<&String> {
        self.memory_map.get_sym(addr)
    }

    pub fn set8(&mut self, addr: u16, byte: u8) {
        match addr {
            DMA => {
                let source = (byte as u16) << 8;
                let mut data = [0; 0xA0];
                for i in 0..0xA0 {
                    data[i] = self.get8(source + i as u16);
                }
                self.devices.lcd_controller().dma(&data);
            }
            HDMA5 => {
                let len = (((byte & 0x7F) as u16) + 1) << 4;
                if b7!(byte) == 1 {
                    self.devices.lcd_controller().vram_hblank_dma(len);
                } else {
                    let source = self.devices.lcd_controller().vram_dma_source();
                    let mut data = Vec::with_capacity(len as usize);
                    for i in 0..len {
                        data.push(self.get8(source + i as u16));
                    }
                    self.devices.lcd_controller().vram_dma(&data);
                }
            }
            _ => {
                self.get_device(addr).set8(addr, byte);
            }
        }
    }

    pub fn set16(&mut self, addr: u16, v: u16) {
        let device = self.get_device(addr);
        device.set8(addr, (v >> 8) as u8);
        device.set8(addr + 1, (v & 0xFF) as u8);
    }

    pub fn get8(&mut self, addr: u16) -> u8 {
        self.get_device(addr).get8(addr)
    }

    pub fn get_arr3(&mut self, addr: u16) -> [u8; 3] {
        [self.get8(addr), self.get8(addr + 1), self.get8(addr + 2)]
    }
}
