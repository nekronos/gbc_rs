use super::Mbc;
use super::MbcInfo;

#[derive(Debug)]
pub struct Mbc5 {
    ram_write_protected: bool,
    rom_bank_0: u8,
    rom_bank_1: u8,
    ram_bank: u8,
    rom_offset: usize,
    ram_offset: usize,
    ram: Box<[u8]>,
}

impl Mbc5 {
    pub fn new(mbc_info: MbcInfo, ram: Option<Box<[u8]>>) -> Mbc5 {
        let ram = if let Some(ram_info) = mbc_info.ram_info {
            ram_info.make_ram(ram)
        } else {
            vec![0; 0].into_boxed_slice()
        };
        Mbc5 {
            ram_write_protected: true,
            rom_bank_0: 0,
            rom_bank_1: 0,
            ram_bank: 0,
            rom_offset: 0,
            ram_offset: 0,
            ram: ram,
        }
    }

    fn update_rom_offset(&mut self) {
        let bank = {
            let upper = (self.rom_bank_1 as usize) << 8;
            let lower = self.rom_bank_0 as usize;
            (upper & 0x100) | lower
        };
        self.rom_offset = bank * 16 * 1024
    }

    fn update_ram_offset(&mut self) {
        self.ram_offset = (self.ram_bank & 0x0f) as usize * 8 * 1024
    }
}

impl Mbc for Mbc5 {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => rom[addr as usize - 0x4000 + self.rom_offset],
            _ => panic!("Address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1fff => self.ram_write_protected = val != 0x0a,
            0x2000...0x2fff => self.rom_bank_0 = val,
            0x3000...0x3fff => self.rom_bank_1 = val,
            0x4000...0x5fff => self.ram_bank = val,
            0x6000...0x7fff => (), // Empty
            _ => panic!("Illegal address: 0x{:x}", addr),
        }
        self.update_rom_offset();
        self.update_ram_offset()
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize - 0xa000 + self.ram_offset]
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if !self.ram_write_protected {
            self.ram[addr as usize - 0xa000 + self.ram_offset] = val
        }
    }

    fn copy_ram(&self) -> Option<Box<[u8]>> {
        if self.ram.len() > 0 {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
